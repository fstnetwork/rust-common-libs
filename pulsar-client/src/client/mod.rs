mod configuration;

use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    sync::Arc,
};

use snafu::ResultExt;
use tokio::sync::oneshot;
use tracing::instrument;
use url::Url;

use pulsar_client_sys::{
    pulsar_client_close, pulsar_client_create, pulsar_client_create_producer_async,
    pulsar_client_free, pulsar_client_get_topic_partitions_async, pulsar_client_subscribe_async,
    pulsar_client_subscribe_multi_topics_async, Client as NativeClient, Consumer as NativeConsumer,
    LogLevel, Producer as NativeProducer, RawResultCode, ResultCode, StringList,
};

use crate::{
    consumer::{Consumer, ConsumerConfiguration},
    error,
    error::Result,
    native::{NativeDrop, NativePointer},
    producer::{MultiTopicProducer, Producer, ProducerConfiguration},
    stl,
};

pub use self::configuration::ClientConfiguration;

unsafe impl NativeDrop for NativeClient {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_client_free;
    const TYPE: &'static str = "Client";
}

#[derive(Debug)]
pub(crate) struct ClientInner(NativePointer<NativeClient>);

impl ClientInner {
    pub(crate) const fn as_ptr(&self) -> *mut NativeClient { self.0.as_ptr() }
}

impl Drop for ClientInner {
    #[instrument(level = "trace", skip(self), fields(r#type = NativeClient::TYPE , pointer = ?self.0))]
    fn drop(&mut self) {
        tracing::trace!("Closing");
        let code = unsafe { pulsar_client_close(&mut *self.0) };
        match ResultCode::from(code) {
            ResultCode::Ok => tracing::trace!("Closed"),
            code => tracing::warn!("Error closing: {code}"),
        }
    }
}

// NOTE:`NativeProducerPointer` is internal use only
type NativeProducerPointer = NativePointer<NativeProducer>;

// SAFETY: the pointer is passed with `tokio::sync::oneshot`
unsafe impl Send for NativeProducerPointer {}

// SAFETY: the pointer is passed with `tokio::sync::oneshot`
unsafe impl Sync for NativeProducerPointer {}

// NOTE:`NativeConsumerPointer` is internal use only
type NativeConsumerPointer = NativePointer<NativeConsumer>;

// SAFETY: the pointer is passed with `tokio::sync::oneshot`
unsafe impl Send for NativeConsumerPointer {}

// SAFETY: the pointer is passed with `tokio::sync::oneshot`
unsafe impl Sync for NativeConsumerPointer {}

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<ClientInner>,
}

impl Client {
    /// # Errors
    /// TODO: document
    pub fn new(server_url: &Url, configuration: &ClientConfiguration) -> Result<Self> {
        let server_url = CString::new(server_url.as_str())
            .context(error::InvalidCStringSnafu)
            .with_context(|_| error::InvalidUrlSnafu { url: server_url.clone() })?;

        unsafe {
            let client = pulsar_client_create(server_url.as_ptr(), configuration.as_ptr());

            Ok(Self { inner: Arc::new(ClientInner(NativePointer::new_unchecked(client))) })
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut NativeClient { self.inner.as_ptr() }

    /// # Errors
    // TODO: document
    pub async fn lookup_partitioned_topic(&self, topic: &str) -> Result<Vec<String>> {
        let (tx, rx) = oneshot::channel::<Result<Vec<String>, ResultCode>>();

        unsafe {
            let topic = CString::new(topic)
                .context(error::InvalidCStringSnafu)
                .with_context(|_| error::InvalidTopicSnafu { topic: topic.to_owned() })?;

            pulsar_client_get_topic_partitions_async(
                self.as_ptr(),
                topic.as_ptr(),
                Some(get_partitions_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        let topics = rx
            .await
            .expect("tx must not drop in FFI; qed")
            .with_context(|_| error::LookupPartitionedTopicSnafu { topic: topic.to_string() })?;

        Ok(topics)
    }

    /// # Errors
    // TODO: document
    #[must_use]
    pub fn multi_topic_producer(&self, configuration: ProducerConfiguration) -> MultiTopicProducer {
        MultiTopicProducer::new(self, configuration)
    }

    /// # Errors
    // TODO: document
    pub async fn create_producer(
        &self,
        topic: &str,
        configuration: &ProducerConfiguration,
    ) -> Result<Producer> {
        let (tx, rx) = oneshot::channel::<Result<NativeProducerPointer, ResultCode>>();

        {
            let topic = CString::new(topic)
                .context(error::InvalidCStringSnafu)
                .with_context(|_| error::InvalidTopicSnafu { topic: topic.to_owned() })?;

            unsafe {
                pulsar_client_create_producer_async(
                    self.as_ptr(),
                    topic.as_ptr(),
                    configuration.as_ptr(),
                    Some(create_producer_callback),
                    Box::into_raw(Box::new(tx)).cast(),
                );
            }
        }

        let producer = rx
            .await
            .expect("tx must not drop in FFI; qed")
            .with_context(|_| error::CreateProducerSnafu { topic: topic.to_string() })?;

        Ok(Producer::new(producer, self.inner.clone()))
    }

    /// # Errors
    /// TODO: document
    pub async fn subscribe(
        &self,
        topic: &str,
        subscription_name: &str,
        configuration: &ConsumerConfiguration,
    ) -> Result<Consumer> {
        let (tx, rx) = oneshot::channel::<Result<NativeConsumerPointer, ResultCode>>();

        {
            let topic = CString::new(topic)
                .context(error::InvalidCStringSnafu)
                .with_context(|_| error::InvalidTopicSnafu { topic: topic.to_owned() })?;
            let subscription_name = CString::new(subscription_name)
                .context(error::InvalidCStringSnafu)
                .with_context(|_| error::InvalidSubscriptionNameSnafu {
                    subscription_name: subscription_name.to_owned(),
                })?;

            unsafe {
                pulsar_client_subscribe_async(
                    self.as_ptr(),
                    topic.as_ptr(),
                    subscription_name.as_ptr(),
                    configuration.as_ptr(),
                    Some(subscribe_callback),
                    Box::into_raw(Box::new(tx)).cast(),
                );
            }
        }

        let consumer = rx.await.expect("tx must not drop in FFI; qed").with_context(|_| {
            error::SubscribeSnafu {
                topics: vec![topic.to_string()],
                subscription_name: subscription_name.to_string(),
            }
        })?;

        Ok(Consumer::new(consumer, self.inner.clone()))
    }

    /// # Errors
    /// TODO: document
    pub async fn multi_subscribe<S, T>(
        &self,
        topics: T,
        subscription_name: &str,
        configuration: &ConsumerConfiguration,
    ) -> Result<Consumer>
    where
        S: AsRef<str>,
        T: IntoIterator<Item = S>,
    {
        let (tx, rx) = oneshot::channel::<Result<NativePointer<NativeConsumer>, ResultCode>>();
        let topics = topics.into_iter().collect::<Vec<_>>();

        {
            let topics = topics
                .iter()
                .map(|topic| {
                    let topic = topic.as_ref();
                    CString::new(topic)
                        .context(error::InvalidCStringSnafu)
                        .with_context(|_| error::InvalidTopicSnafu { topic: topic.to_owned() })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let subscription_name = CString::new(subscription_name)
                .context(error::InvalidCStringSnafu)
                .with_context(|_| error::InvalidSubscriptionNameSnafu {
                    subscription_name: subscription_name.to_owned(),
                })?;

            let mut topic_ptrs = topics.iter().map(|t| t.as_ptr()).collect::<Vec<_>>();
            let count = i32::try_from(topic_ptrs.len())
                .with_context(|_| error::NumberOutOfRangeSnafu {
                    number: topic_ptrs.len().to_string(),
                    range: format!("[0,{}]", i32::MAX),
                })
                .context(error::SubscribeTooManyTopicsSnafu)?;
            unsafe {
                pulsar_client_subscribe_multi_topics_async(
                    self.as_ptr(),
                    topic_ptrs.as_mut_ptr(),
                    count,
                    subscription_name.as_ptr(),
                    configuration.as_ptr(),
                    Some(subscribe_callback),
                    Box::into_raw(Box::new(tx)).cast(),
                );
            }
        }

        let inner = rx.await.expect("tx must not drop in FFI; qed").with_context(|_| {
            error::SubscribeSnafu {
                topics: topics
                    .into_iter()
                    .map(|topic| topic.as_ref().to_owned())
                    .collect::<Vec<_>>(),
                subscription_name: subscription_name.to_string(),
            }
        })?;

        Ok(Consumer::new(inner, self.inner.clone()))
    }
}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Client {}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for Client {}

// SAFETY: the complexity is caused by tracing macro
#[allow(clippy::cognitive_complexity)]
pub(crate) unsafe extern "C" fn log(
    level: LogLevel,
    file: *const c_char,
    line: c_int,
    message: *const c_char,
    _ctx: *mut c_void,
) {
    let file = unsafe { CStr::from_ptr(file) }.to_string_lossy();
    let location = format!("{file}.cc:{line}");
    let message = unsafe { CStr::from_ptr(message) }.to_string_lossy().into_owned();

    match level {
        LogLevel::pulsar_DEBUG => tracing::debug!(location, message),
        LogLevel::pulsar_INFO => tracing::info!(location, message),
        LogLevel::pulsar_WARN => tracing::warn!(location, message),
        LogLevel::pulsar_ERROR => tracing::error!(location, message),
    }
}

unsafe extern "C" fn subscribe_callback(
    result: RawResultCode,
    consumer: *mut NativeConsumer,
    ctx: *mut c_void,
) {
    let tx = unsafe {
        Box::from_raw(ctx.cast::<oneshot::Sender<Result<NativeConsumerPointer, ResultCode>>>())
    };

    let result =
        ResultCode::from(result).err_or_else(|| unsafe { NativePointer::new_unchecked(consumer) });

    let _send = tx.send(result);
}

unsafe extern "C" fn create_producer_callback(
    result: RawResultCode,
    producer: *mut NativeProducer,
    ctx: *mut c_void,
) {
    let tx = unsafe {
        Box::from_raw(ctx.cast::<oneshot::Sender<Result<NativeProducerPointer, ResultCode>>>())
    };

    let result =
        ResultCode::from(result).err_or_else(|| unsafe { NativePointer::new_unchecked(producer) });

    let _send = tx.send(result);
}

unsafe extern "C" fn get_partitions_callback(
    result: RawResultCode,
    partitions: *mut StringList,
    ctx: *mut c_void,
) {
    let tx =
        unsafe { Box::from_raw(ctx.cast::<oneshot::Sender<Result<Vec<String>, ResultCode>>>()) };

    let result = ResultCode::from(result).err_or_else(|| {
        let partitions = unsafe { NativePointer::new_unchecked(partitions) };
        stl::convert_string_list_from_ffi(&partitions)
    });

    let _send = tx.send(result);
}
