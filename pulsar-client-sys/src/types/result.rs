use std::{error::Error, ffi::CStr, fmt};

use crate::{bindings, types::RawResultCode};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResultCode {
    Ok = 0,
    UnknownError = 1,
    InvalidConfiguration = 2,
    Timeout = 3,
    LookupError = 4,
    ConnectError = 5,
    ReadError = 6,
    AuthenticationError = 7,
    AuthorizationError = 8,
    ErrorGettingAuthenticationData = 9,
    BrokerMetadataError = 10,
    BrokerPersistenceError = 11,
    ChecksumError = 12,
    ConsumerBusy = 13,
    NotConnected = 14,
    AlreadyClosed = 15,
    InvalidMessage = 16,
    ConsumerNotInitialized = 17,
    ProducerNotInitialized = 18,
    ProducerBusy = 19,
    TooManyLookupRequestException = 20,
    InvalidTopicName = 21,
    InvalidUrl = 22,
    ServiceUnitNotReady = 23,
    OperationNotSupported = 24,
    ProducerBlockedQuotaExceededError = 25,
    ProducerBlockedQuotaExceededException = 26,
    ProducerQueueIsFull = 27,
    MessageTooBig = 28,
    TopicNotFound = 29,
    SubscriptionNotFound = 30,
    ConsumerNotFound = 31,
    UnsupportedVersionError = 32,
    TopicTerminated = 33,
    CryptoError = 34,
    IncompatibleSchema = 35,
    ConsumerAssignError = 36,
    CumulativeAcknowledgementNotAllowedError = 37,
    TransactionCoordinatorNotFoundError = 38,
    InvalidTxnStatusError = 39,
    NotAllowedError = 40,
    TransactionConflict = 41,
    TransactionNotFound = 42,
    ProducerFenced = 43,
    MemoryBufferIsFull = 44,
    Interrupted = 45,
}

impl ResultCode {
    pub fn err_or<U>(self, v: U) -> Result<U, Self> {
        match self {
            Self::Ok => Ok(v),
            code => Err(code),
        }
    }

    pub fn err_or_else<U, F>(self, op: F) -> Result<U, Self>
    where
        F: FnOnce() -> U,
    {
        match self {
            Self::Ok => Ok(op()),
            code => Err(code),
        }
    }
}

impl From<ResultCode> for RawResultCode {
    fn from(result: ResultCode) -> Self {
        match result {
            ResultCode::Ok => Self::pulsar_result_Ok,
            ResultCode::UnknownError => Self::pulsar_result_UnknownError,
            ResultCode::InvalidConfiguration => Self::pulsar_result_InvalidConfiguration,
            ResultCode::Timeout => Self::pulsar_result_Timeout,
            ResultCode::LookupError => Self::pulsar_result_LookupError,
            ResultCode::ConnectError => Self::pulsar_result_ConnectError,
            ResultCode::ReadError => Self::pulsar_result_ReadError,
            ResultCode::AuthenticationError => Self::pulsar_result_AuthenticationError,
            ResultCode::AuthorizationError => Self::pulsar_result_AuthorizationError,
            ResultCode::ErrorGettingAuthenticationData => {
                Self::pulsar_result_ErrorGettingAuthenticationData
            }
            ResultCode::BrokerMetadataError => Self::pulsar_result_BrokerMetadataError,
            ResultCode::BrokerPersistenceError => Self::pulsar_result_BrokerPersistenceError,
            ResultCode::ChecksumError => Self::pulsar_result_ChecksumError,
            ResultCode::ConsumerBusy => Self::pulsar_result_ConsumerBusy,
            ResultCode::NotConnected => Self::pulsar_result_NotConnected,
            ResultCode::AlreadyClosed => Self::pulsar_result_AlreadyClosed,
            ResultCode::InvalidMessage => Self::pulsar_result_InvalidMessage,
            ResultCode::ConsumerNotInitialized => Self::pulsar_result_ConsumerNotInitialized,
            ResultCode::ProducerNotInitialized => Self::pulsar_result_ProducerNotInitialized,
            ResultCode::ProducerBusy => Self::pulsar_result_ProducerBusy,
            ResultCode::TooManyLookupRequestException => {
                Self::pulsar_result_TooManyLookupRequestException
            }
            ResultCode::InvalidTopicName => Self::pulsar_result_InvalidTopicName,
            ResultCode::InvalidUrl => Self::pulsar_result_InvalidUrl,
            ResultCode::ServiceUnitNotReady => Self::pulsar_result_ServiceUnitNotReady,
            ResultCode::OperationNotSupported => Self::pulsar_result_OperationNotSupported,
            ResultCode::ProducerBlockedQuotaExceededError => {
                Self::pulsar_result_ProducerBlockedQuotaExceededError
            }
            ResultCode::ProducerBlockedQuotaExceededException => {
                Self::pulsar_result_ProducerBlockedQuotaExceededException
            }
            ResultCode::ProducerQueueIsFull => Self::pulsar_result_ProducerQueueIsFull,
            ResultCode::MessageTooBig => Self::pulsar_result_MessageTooBig,
            ResultCode::TopicNotFound => Self::pulsar_result_TopicNotFound,
            ResultCode::SubscriptionNotFound => Self::pulsar_result_SubscriptionNotFound,
            ResultCode::ConsumerNotFound => Self::pulsar_result_ConsumerNotFound,
            ResultCode::UnsupportedVersionError => Self::pulsar_result_UnsupportedVersionError,
            ResultCode::TopicTerminated => Self::pulsar_result_TopicTerminated,
            ResultCode::CryptoError => Self::pulsar_result_CryptoError,
            ResultCode::IncompatibleSchema => Self::pulsar_result_IncompatibleSchema,
            ResultCode::ConsumerAssignError => Self::pulsar_result_ConsumerAssignError,
            ResultCode::CumulativeAcknowledgementNotAllowedError => {
                Self::pulsar_result_CumulativeAcknowledgementNotAllowedError
            }
            ResultCode::TransactionCoordinatorNotFoundError => {
                Self::pulsar_result_TransactionCoordinatorNotFoundError
            }
            ResultCode::InvalidTxnStatusError => Self::pulsar_result_InvalidTxnStatusError,
            ResultCode::NotAllowedError => Self::pulsar_result_NotAllowedError,
            ResultCode::TransactionConflict => Self::pulsar_result_TransactionConflict,
            ResultCode::TransactionNotFound => Self::pulsar_result_TransactionNotFound,
            ResultCode::ProducerFenced => Self::pulsar_result_ProducerFenced,
            ResultCode::MemoryBufferIsFull => Self::pulsar_result_MemoryBufferIsFull,
            ResultCode::Interrupted => Self::pulsar_result_Interrupted,
        }
    }
}

impl From<RawResultCode> for ResultCode {
    fn from(result: RawResultCode) -> Self {
        match result {
            RawResultCode::pulsar_result_Ok => Self::Ok,
            RawResultCode::pulsar_result_UnknownError => Self::UnknownError,
            RawResultCode::pulsar_result_InvalidConfiguration => Self::InvalidConfiguration,
            RawResultCode::pulsar_result_Timeout => Self::Timeout,
            RawResultCode::pulsar_result_LookupError => Self::LookupError,
            RawResultCode::pulsar_result_ConnectError => Self::ConnectError,
            RawResultCode::pulsar_result_ReadError => Self::ReadError,
            RawResultCode::pulsar_result_AuthenticationError => Self::AuthenticationError,
            RawResultCode::pulsar_result_AuthorizationError => Self::AuthorizationError,
            RawResultCode::pulsar_result_ErrorGettingAuthenticationData => {
                Self::ErrorGettingAuthenticationData
            }
            RawResultCode::pulsar_result_BrokerMetadataError => Self::BrokerMetadataError,
            RawResultCode::pulsar_result_BrokerPersistenceError => Self::BrokerPersistenceError,
            RawResultCode::pulsar_result_ChecksumError => Self::ChecksumError,
            RawResultCode::pulsar_result_ConsumerBusy => Self::ConsumerBusy,
            RawResultCode::pulsar_result_NotConnected => Self::NotConnected,
            RawResultCode::pulsar_result_AlreadyClosed => Self::AlreadyClosed,
            RawResultCode::pulsar_result_InvalidMessage => Self::InvalidMessage,
            RawResultCode::pulsar_result_ConsumerNotInitialized => Self::ConsumerNotInitialized,
            RawResultCode::pulsar_result_ProducerNotInitialized => Self::ProducerNotInitialized,
            RawResultCode::pulsar_result_ProducerBusy => Self::ProducerBusy,
            RawResultCode::pulsar_result_TooManyLookupRequestException => {
                Self::TooManyLookupRequestException
            }
            RawResultCode::pulsar_result_InvalidTopicName => Self::InvalidTopicName,
            RawResultCode::pulsar_result_InvalidUrl => Self::InvalidUrl,
            RawResultCode::pulsar_result_ServiceUnitNotReady => Self::ServiceUnitNotReady,
            RawResultCode::pulsar_result_OperationNotSupported => Self::OperationNotSupported,
            RawResultCode::pulsar_result_ProducerBlockedQuotaExceededError => {
                Self::ProducerBlockedQuotaExceededError
            }
            RawResultCode::pulsar_result_ProducerBlockedQuotaExceededException => {
                Self::ProducerBlockedQuotaExceededException
            }
            RawResultCode::pulsar_result_ProducerQueueIsFull => Self::ProducerQueueIsFull,
            RawResultCode::pulsar_result_MessageTooBig => Self::MessageTooBig,
            RawResultCode::pulsar_result_TopicNotFound => Self::TopicNotFound,
            RawResultCode::pulsar_result_SubscriptionNotFound => Self::SubscriptionNotFound,
            RawResultCode::pulsar_result_ConsumerNotFound => Self::ConsumerNotFound,
            RawResultCode::pulsar_result_UnsupportedVersionError => Self::UnsupportedVersionError,
            RawResultCode::pulsar_result_TopicTerminated => Self::TopicTerminated,
            RawResultCode::pulsar_result_CryptoError => Self::CryptoError,
            RawResultCode::pulsar_result_IncompatibleSchema => Self::IncompatibleSchema,
            RawResultCode::pulsar_result_ConsumerAssignError => Self::ConsumerAssignError,
            RawResultCode::pulsar_result_CumulativeAcknowledgementNotAllowedError => {
                Self::CumulativeAcknowledgementNotAllowedError
            }
            RawResultCode::pulsar_result_TransactionCoordinatorNotFoundError => {
                Self::TransactionCoordinatorNotFoundError
            }
            RawResultCode::pulsar_result_InvalidTxnStatusError => Self::InvalidTxnStatusError,
            RawResultCode::pulsar_result_NotAllowedError => Self::NotAllowedError,
            RawResultCode::pulsar_result_TransactionConflict => Self::TransactionConflict,
            RawResultCode::pulsar_result_TransactionNotFound => Self::TransactionNotFound,
            RawResultCode::pulsar_result_ProducerFenced => Self::ProducerFenced,
            RawResultCode::pulsar_result_MemoryBufferIsFull => Self::MemoryBufferIsFull,
            RawResultCode::pulsar_result_Interrupted => Self::Interrupted,
        }
    }
}

impl Error for ResultCode {}

impl fmt::Display for ResultCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = RawResultCode::from(*self);
        let c_str = unsafe { CStr::from_ptr(bindings::pulsar_result_str(result)) };

        f.write_str(&c_str.to_string_lossy())
    }
}
