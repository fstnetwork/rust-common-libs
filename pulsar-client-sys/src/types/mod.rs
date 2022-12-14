mod result;

use crate::bindings;

pub use self::result::ResultCode;

pub type StringMap = bindings::pulsar_string_map_t;
pub type StringList = bindings::pulsar_string_list_t;

pub type Authentication = bindings::pulsar_authentication_t;
pub type LogLevel = bindings::pulsar_logger_level_t;
pub type RawResultCode = bindings::pulsar_result;

pub type Client = bindings::pulsar_client_t;
pub type ClientConfiguration = bindings::pulsar_client_configuration_t;

pub type Producer = bindings::pulsar_producer_t;
pub type ProducerConfiguration = bindings::pulsar_producer_configuration_t;

pub type Consumer = bindings::pulsar_consumer_t;
pub type ConsumerConfiguration = bindings::pulsar_consumer_configuration_t;
pub type ConsumerCryptoFailureAction = bindings::pulsar_consumer_crypto_failure_action;
pub type ConsumerType = bindings::pulsar_consumer_type;
pub type InitialPosition = bindings::initial_position;

pub type Reader = bindings::pulsar_reader_t;
pub type ReaderConfiguration = bindings::pulsar_reader_configuration_t;

pub type Message = bindings::pulsar_message_t;
pub type MessageId = bindings::pulsar_message_id_t;
