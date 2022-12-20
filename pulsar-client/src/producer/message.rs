use std::collections::HashMap;

/// message data that will be sent on a topic
///
/// generated from the [`SerializeMessage`] trait or [`MessageBuilder`]
///
/// this is actually a subset of the fields of a message, because batching,
/// compression and encryption should be handled by the producer
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// serialized data
    pub payload: Vec<u8>,

    /// user defined properties
    pub properties: HashMap<String, String>,

    /// key to decide partition for the message
    pub partition_key: Option<String>,

    /// key to decide partition for the message
    pub ordering_key: Option<Vec<u8>>,

    /// Override namespace's replication
    pub replicate_to: Vec<String>,

    /// the timestamp that this event occurs. it is typically set by
    /// applications. if this field is omitted, `publish_time` can be used
    /// for the purpose of `event_time`.
    pub event_time: Option<u64>,

    /// current version of the schema
    pub schema_version: Option<Vec<u8>>,
}
