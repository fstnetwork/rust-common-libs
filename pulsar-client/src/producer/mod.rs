mod configuration;
mod message;
mod multi_topic;

use std::{ffi::c_void, sync::Arc};

use pulsar_client_sys::{
    pulsar_producer_close, pulsar_producer_free, pulsar_producer_is_connected,
    pulsar_producer_send_async, MessageId as NativeMessageId, Producer as NativeProducer,
    RawResultCode, ResultCode,
};
use snafu::ResultExt;
use tokio::{sync::oneshot, task};
use tracing::instrument;

use crate::{
    client::ClientInner,
    error,
    error::Result,
    message::{MessageId, SerializeMessage},
    native::{NativeDrop, NativePointer},
};

pub use self::{
    configuration::ProducerConfiguration, message::Message, multi_topic::MultiTopicProducer,
};

unsafe impl NativeDrop for NativeProducer {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_producer_free;
    const TYPE: &'static str = "Producer";
}

#[derive(Debug)]
pub struct Producer {
    inner: NativePointer<NativeProducer>,
    _client: Arc<ClientInner>,
}

impl Producer {
    pub(crate) fn new(inner: NativePointer<NativeProducer>, client: Arc<ClientInner>) -> Self {
        Self { inner, _client: client }
    }

    pub(crate) const fn as_ptr(&self) -> *mut NativeProducer { self.inner.as_ptr() }

    /// # Errors
    // TODO: document
    pub async fn send<Message>(&self, message: Message) -> Result<MessageId>
    where
        Message: SerializeMessage,
    {
        let (tx, rx) = oneshot::channel::<Result<MessageId, ResultCode>>();

        let message = crate::message::Message::try_from(Message::serialize_message(message)?)?;
        unsafe {
            pulsar_producer_send_async(
                self.as_ptr(),
                message.as_ptr(),
                Some(send_callback),
                Box::into_raw(Box::new(tx)).cast(),
            );
        }

        let message_id = rx
            .await
            .expect("tx must not drop in FFI; qed")
            .with_context(|_| error::ReceiveSnafu)?;

        Ok(message_id)
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        task::block_in_place(|| unsafe { pulsar_producer_is_connected(self.as_ptr()) != 0 })
    }
}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Producer {}

// SAFETY: client is thread safe
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for Producer {}

impl Drop for Producer {
    #[instrument(
        level = "trace",
        skip(self),
        fields(r#type = NativeProducer::TYPE , pointer = ?self.inner)
    )]
    fn drop(&mut self) {
        tracing::trace!("Closing");
        let code = unsafe { pulsar_producer_close(&mut *self.inner) };
        match ResultCode::from(code) {
            ResultCode::Ok => tracing::trace!("Closed"),
            code => tracing::warn!("Error closing: {code}"),
        }
    }
}

unsafe extern "C" fn send_callback(
    result: RawResultCode,
    message_id: *mut NativeMessageId,
    ctx: *mut c_void,
) {
    let tx = unsafe { Box::from_raw(ctx.cast::<oneshot::Sender<Result<MessageId, ResultCode>>>()) };

    let message_id = ResultCode::from(result).err_or_else(|| {
        let message_id = unsafe { NativePointer::new_unchecked(message_id) };
        MessageId::new(message_id)
    });

    let _send = tx.send(message_id);
}
