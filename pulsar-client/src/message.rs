use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
    fmt, slice,
    string::FromUtf8Error,
};

use snafu::ResultExt;

use pulsar_client_sys::{
    pulsar_message_create, pulsar_message_free, pulsar_message_get_data, pulsar_message_get_length,
    pulsar_message_get_message_id, pulsar_message_get_partitionKey, pulsar_message_get_properties,
    pulsar_message_get_topic_name, pulsar_message_has_partition_key, pulsar_message_id_free,
    pulsar_message_id_str, pulsar_message_set_content, pulsar_message_set_event_timestamp,
    pulsar_message_set_ordering_key, pulsar_message_set_partition_key, pulsar_message_set_property,
    Message as NativeMessage, MessageId as NativeMessageId,
};

use crate::{
    error,
    error::Error,
    native::{NativeDrop, NativePointer},
    producer, stl, utils,
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
    pub(crate) const fn new(inner: NativePointer<NativeMessageId>) -> Self { Self { inner } }

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

impl Default for Message {
    fn default() -> Self {
        let ptr = unsafe {
            let message = pulsar_message_create();
            NativePointer::new_unchecked(message)
        };

        Self::from(ptr)
    }
}

impl From<NativePointer<NativeMessage>> for Message {
    fn from(ptr: NativePointer<NativeMessage>) -> Self { Self { inner: ptr } }
}

impl Message {
    pub(crate) const fn new(inner: NativePointer<NativeMessage>) -> Self { Self { inner } }

    pub(crate) const fn as_ptr(&self) -> *mut NativeMessage { self.inner.as_ptr() }

    #[must_use]
    pub fn with_content(data: &[u8]) -> Self {
        let inner = unsafe {
            let message = pulsar_message_create();
            pulsar_message_set_content(message, data.as_ptr().cast::<c_void>(), data.len());
            NativePointer::new_unchecked(message)
        };

        Self::from(inner)
    }

    /// # Errors
    /// return an error if the supplied bytes contain an internal 0 byte
    pub fn set_property(&self, name: &str, value: &str) -> Result<(), Error> {
        let name = CString::new(name).context(error::InvalidCStringSnafu)?;
        let value = CString::new(value).context(error::InvalidCStringSnafu)?;

        unsafe {
            pulsar_message_set_property(self.as_ptr(), name.as_ptr(), value.as_ptr());
        }

        Ok(())
    }

    #[must_use]
    pub fn get_properties(&self) -> HashMap<String, String> {
        let properties =
            unsafe { NativePointer::new_unchecked(pulsar_message_get_properties(self.as_ptr())) };

        stl::convert_string_map_from_ffi(&properties)
    }

    pub fn set_content(&self, data: &[u8]) {
        unsafe {
            pulsar_message_set_content(self.as_ptr(), data.as_ptr().cast::<c_void>(), data.len());
        }
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

        MessageId::new(inner)
    }

    #[must_use]
    pub fn get_partition_key(&self) -> Option<String> {
        let has_partition_key = unsafe { pulsar_message_has_partition_key(self.as_ptr()) };

        if utils::convert_bool_from_ffi_c_int(has_partition_key) {
            let c_str = unsafe { CStr::from_ptr(pulsar_message_get_partitionKey(self.as_ptr())) };
            Some(c_str.to_string_lossy().into_owned())
        } else {
            None
        }
    }

    /// # Errors
    /// return an error if the supplied bytes contain an internal 0 byte
    pub fn set_partition_key(&self, partition_key: &str) -> Result<(), Error> {
        let partition_key = CString::new(partition_key).context(error::InvalidCStringSnafu)?;

        unsafe {
            pulsar_message_set_partition_key(self.as_ptr(), partition_key.as_ptr());
        }

        Ok(())
    }

    pub fn set_event_time(&self, event_timestamp: u64) {
        unsafe {
            pulsar_message_set_event_timestamp(self.as_ptr(), event_timestamp);
        }
    }

    /// # Errors
    /// return an error if the supplied bytes contain an internal 0 byte
    pub fn set_ordering_key(&self, ordering_key: &str) -> Result<(), Error> {
        let ordering_key = CString::new(ordering_key).context(error::InvalidCStringSnafu)?;

        unsafe {
            pulsar_message_set_ordering_key(self.as_ptr(), ordering_key.as_ptr());
        }

        Ok(())
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

impl TryFrom<producer::Message> for Message {
    type Error = Error;

    fn try_from(
        producer::Message {
            payload,
            properties,
            partition_key,
            ordering_key,
            replicate_to: _, // TODO:
            event_time,
            schema_version,
        }: producer::Message,
    ) -> Result<Self, Self::Error> {
        let message = Self::default();

        message.set_content(payload.as_ref());
        for (name, value) in properties {
            message.set_property(&name, &value)?;
        }

        if let Some(partition_key) = partition_key {
            message.set_partition_key(&partition_key)?;
        }

        if let Some(ordering_key) = ordering_key {
            message.set_ordering_key(&ordering_key)?;
        }

        if let Some(event_time) = event_time {
            message.set_event_time(event_time);
        }

        if let Some(_schema_version) = schema_version {
            // TODO: implement `set_schema_version`
            // message.set_schema_version(&schema_version);
        }

        Ok(message)
    }
}

/// Helper trait for consumer deserialization
pub trait DeserializeMessage {
    /// type produced from the message
    type Output: Sized;
    /// deserialize method that will be called by the consumer
    fn deserialize_message(message: &Message) -> Self::Output;
}

impl DeserializeMessage for Vec<u8> {
    type Output = Self;

    fn deserialize_message(message: &Message) -> Self::Output { message.get_data() }
}

impl DeserializeMessage for String {
    type Output = Result<Self, FromUtf8Error>;

    fn deserialize_message(message: &Message) -> Self::Output {
        Self::from_utf8(message.get_data())
    }
}

/// Helper trait for message serialization
pub trait SerializeMessage {
    /// serialize method that will be called by the producer
    /// # Errors
    /// returns error while failed to serializing message
    fn serialize_message(input: Self) -> Result<producer::Message, Error>;
}

impl SerializeMessage for producer::Message {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> { Ok(input) }
}

impl SerializeMessage for () {
    fn serialize_message(_input: Self) -> Result<producer::Message, Error> {
        Ok(producer::Message::default())
    }
}

impl SerializeMessage for &[u8] {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> {
        Ok(producer::Message { payload: input.to_vec(), ..Default::default() })
    }
}

impl SerializeMessage for Vec<u8> {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> {
        Ok(producer::Message { payload: input, ..Default::default() })
    }
}

impl SerializeMessage for String {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> {
        let payload = input.into_bytes();
        Ok(producer::Message { payload, ..Default::default() })
    }
}

impl SerializeMessage for &String {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> {
        let payload = input.as_bytes().to_vec();
        Ok(producer::Message { payload, ..Default::default() })
    }
}

impl SerializeMessage for &str {
    fn serialize_message(input: Self) -> Result<producer::Message, Error> {
        let payload = input.as_bytes().to_vec();
        Ok(producer::Message { payload, ..Default::default() })
    }
}
