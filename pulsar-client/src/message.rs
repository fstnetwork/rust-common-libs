use std::{collections::HashMap, ffi::CStr, fmt, slice};

use pulsar_client_sys::{
    pulsar_message_free, pulsar_message_get_data, pulsar_message_get_length,
    pulsar_message_get_message_id, pulsar_message_get_partitionKey, pulsar_message_get_properties,
    pulsar_message_get_topic_name, pulsar_message_has_partition_key, pulsar_message_id_free,
    pulsar_message_id_str, Message as NativeMessage, MessageId as NativeMessageId,
};

use crate::{
    native::{NativeDrop, NativePointer},
    stl, util,
};

unsafe impl NativeDrop for NativeMessageId {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_message_id_free;
    const TYPE: &'static str = "MessageId";
}

unsafe impl NativeDrop for NativeMessage {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_message_free;
    const TYPE: &'static str = "Message";
}

#[derive(Debug)]
pub struct MessageId {
    inner: NativePointer<NativeMessageId>,
}

impl MessageId {
    pub(crate) const fn as_ptr(&self) -> *mut NativeMessageId { self.inner.as_ptr() }
}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for MessageId {}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for MessageId {}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_str = unsafe { CStr::from_ptr(pulsar_message_id_str(self.as_ptr())) };

        c_str.to_string_lossy().fmt(f)
    }
}

#[derive(Debug)]
pub struct Message {
    inner: NativePointer<NativeMessage>,
}

impl Message {
    pub(crate) const fn new(inner: NativePointer<NativeMessage>) -> Self { Self { inner } }

    pub(crate) const fn as_ptr(&self) -> *mut NativeMessage { self.inner.as_ptr() }

    #[must_use]
    pub fn get_properties(&self) -> HashMap<String, String> {
        let properties =
            unsafe { NativePointer::new_unchecked(pulsar_message_get_properties(self.as_ptr())) };

        stl::convert_string_map_from_ffi(&properties)
    }

    #[must_use]
    pub fn get_data(&self) -> Vec<u8> {
        let data = unsafe {
            let ptr = pulsar_message_get_data(self.as_ptr());
            let len = pulsar_message_get_length(self.as_ptr());

            slice::from_raw_parts(ptr as *mut u8, len as usize)
        };

        data.to_vec()
    }

    // getDataAsString
    // getKeyValueData

    #[must_use]
    pub fn get_message_id(&self) -> MessageId {
        let inner =
            unsafe { NativePointer::new_unchecked(pulsar_message_get_message_id(self.as_ptr())) };

        MessageId { inner }
    }

    #[must_use]
    pub fn get_partition_key(&self) -> Option<String> {
        let has_partition_key = unsafe { pulsar_message_has_partition_key(self.as_ptr()) };

        if util::convert_bool_from_ffi_c_int(has_partition_key) {
            let c_str = unsafe { CStr::from_ptr(pulsar_message_get_partitionKey(self.as_ptr())) };
            Some(c_str.to_string_lossy().into_owned())
        } else {
            None
        }
    }

    // getOrderingKey
    // getPublishTimestamp
    // getEventTimestamp

    #[must_use]
    pub fn get_topic_name(&self) -> String {
        let topic_name = unsafe { CStr::from_ptr(pulsar_message_get_topic_name(self.as_ptr())) };

        topic_name.to_string_lossy().to_string()
    }

    // getRedeliveryCount
    // hasSchemaVersion
}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Message {}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for Message {}
