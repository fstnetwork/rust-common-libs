mod configuration;

use std::{ffi::c_void, sync::Arc};

use snafu::ResultExt;
use tokio::{
    sync::{mpsc, oneshot},
    task,
};
use tracing::instrument;

use pulsar_client_sys::{
    pulsar_consumer_acknowledge_async, pulsar_consumer_acknowledge_async_id,
    pulsar_consumer_acknowledge_cumulative_async, pulsar_consumer_acknowledge_cumulative_async_id,
    pulsar_consumer_close, pulsar_consumer_free, pulsar_consumer_negative_acknowledge,
    pulsar_consumer_negative_acknowledge_id, pulsar_consumer_receive_async,
    Consumer as NativeConsumer, Message as NativeMessage, RawResultCode, ResultCode,
};

use crate::{
    client::ClientInner,
    error,
    error::Result,
    message::{Message, MessageId},
    native::{NativeDrop, NativePointer},
};

pub use self::configuration::{ConsumerConfiguration, ConsumerType, InitialPosition};

unsafe impl NativeDrop for NativeConsumer {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_consumer_free;
    const TYPE: &'static str = "Consumer";
}

#[derive(Debug)]
pub struct Consumer {
    inner: NativePointer<NativeConsumer>,
    tx: mpsc::UnboundedSender<Result<Message, ResultCode>>,
    rx: mpsc::UnboundedReceiver<Result<Message, ResultCode>>,
    pending_receiving: bool,
    _client: Arc<ClientInner>,
}

impl Consumer {
    pub(crate) fn new(inner: NativePointer<NativeConsumer>, client: Arc<ClientInner>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self { inner, tx, rx, pending_receiving: false, _client: client }
    }

    pub(crate) const fn as_ptr(&self) -> *mut NativeConsumer { self.inner.as_ptr() }

    // getTopic
    // getSubscriptionName
    // unsubscribe
    // receive

    /// # Errors
    /// TODO: document
    pub async fn receive(&mut self) -> Result<Message> {
        if !self.pending_receiving {
            let tx = self.tx.clone();
            unsafe {
                pulsar_consumer_receive_async(
                    self.as_ptr(),
                    Some(receive_callback),
                    Box::into_raw(Box::new(tx)).cast(),
                );
            }
            self.pending_receiving = true;
        }

        let message = self
            .rx
            .recv()
            .await
            .expect("tx must not drop in FFI; qed")
            .with_context(|_| error::ReceiveSnafu)?;
        self.pending_receiving = false;

        Ok(message)
    }

    // batchReceive

    /// # Errors
    /// TODO: document
    pub async fn acknowledge(&mut self, message: &Message) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<(), ResultCode>>();

        unsafe {
            pulsar_consumer_acknowledge_async(
                self.as_ptr(),
                message.as_ptr(),
                Some(result_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        rx.await.expect("tx must not drop in FFI; qed").with_context(|_| error::AcknowledgeSnafu)
    }

    /// # Errors
    /// TODO: document
    pub async fn acknowledge_id(&mut self, message_id: &MessageId) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<(), ResultCode>>();

        unsafe {
            pulsar_consumer_acknowledge_async_id(
                self.as_ptr(),
                message_id.as_ptr(),
                Some(result_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        rx.await.expect("tx must not drop in FFI; qed").with_context(|_| error::AcknowledgeSnafu)
    }

    /// # Errors
    /// TODO: document
    pub async fn acknowledge_cumulative(&mut self, message: &Message) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<(), ResultCode>>();

        unsafe {
            pulsar_consumer_acknowledge_cumulative_async(
                self.as_ptr(),
                message.as_ptr(),
                Some(result_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        rx.await.expect("tx must not drop in FFI; qed").with_context(|_| error::AcknowledgeSnafu)
    }

    /// # Errors
    /// TODO: document
    pub async fn acknowledge_cumulative_id(&mut self, message_id: &MessageId) -> Result<()> {
        let (tx, rx) = oneshot::channel::<Result<(), ResultCode>>();

        unsafe {
            pulsar_consumer_acknowledge_cumulative_async_id(
                self.as_ptr(),
                message_id.as_ptr(),
                Some(result_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        rx.await.expect("tx must not drop in FFI; qed").with_context(|_| error::AcknowledgeSnafu)
    }

    pub fn negative_acknowledge(&mut self, message: &Message) {
        task::block_in_place(|| unsafe {
            pulsar_consumer_negative_acknowledge(self.as_ptr(), message.as_ptr());
        });
    }

    pub fn negative_acknowledge_id(&mut self, message_id: &MessageId) {
        task::block_in_place(|| unsafe {
            pulsar_consumer_negative_acknowledge_id(self.as_ptr(), message_id.as_ptr());
        });
    }

    // pauseMessageListener
    // resumeMessageListener
    // redeliverUnacknowledgedMessages
    // getBrokerConsumerStats
    // seek
    // isConnected
    // getLastMessageId
}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Consumer {}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for Consumer {}

impl Drop for Consumer {
    #[instrument(
        level = "trace",
        skip(self),
        fields(r#type = NativeConsumer::TYPE , pointer = ?self.inner)
    )]
    fn drop(&mut self) {
        tracing::trace!("Closing");
        let code = unsafe { pulsar_consumer_close(&mut *self.inner) };
        match ResultCode::from(code) {
            ResultCode::Ok => {
                tracing::trace!("Closed");
            }
            code => {
                tracing::warn!("Error closing: {code}");
            }
        }
    }
}

unsafe extern "C" fn receive_callback(
    result: RawResultCode,
    message: *mut NativeMessage,
    ctx: *mut c_void,
) {
    let tx =
        unsafe { Box::from_raw(ctx.cast::<mpsc::UnboundedSender<Result<Message, ResultCode>>>()) };

    let result = ResultCode::from(result).err_or_else(|| {
        let message = unsafe { NativePointer::new_unchecked(message) };
        Message::new(message)
    });

    let _send = tx.send(result);
}

unsafe extern "C" fn result_callback(result: RawResultCode, ctx: *mut c_void) {
    let tx = unsafe { Box::from_raw(ctx.cast::<oneshot::Sender<Result<(), ResultCode>>>()) };

    let result = ResultCode::from(result).err_or(());

    let _send = tx.send(result);
}
