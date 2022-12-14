use std::{
    ffi::{CStr, CString},
    time::Duration,
};

use snafu::ResultExt;

use pulsar_client_sys::{
    pulsar_configure_get_ack_grouping_time_ms,
    pulsar_configure_get_negative_ack_redelivery_delay_ms,
    pulsar_configure_set_ack_grouping_time_ms,
    pulsar_configure_set_negative_ack_redelivery_delay_ms, pulsar_consumer_configuration_create,
    pulsar_consumer_configuration_free, pulsar_consumer_configuration_get_consumer_type,
    pulsar_consumer_configuration_get_max_pending_chunked_message,
    pulsar_consumer_configuration_get_receiver_queue_size,
    pulsar_consumer_configuration_set_consumer_type,
    pulsar_consumer_configuration_set_max_pending_chunked_message,
    pulsar_consumer_configuration_set_receiver_queue_size, pulsar_consumer_get_consumer_name,
    pulsar_consumer_get_subscription_initial_position,
    pulsar_consumer_get_unacked_messages_timeout_ms, pulsar_consumer_is_read_compacted,
    pulsar_consumer_set_consumer_name, pulsar_consumer_set_read_compacted,
    pulsar_consumer_set_subscription_initial_position,
    pulsar_consumer_set_unacked_messages_timeout_ms,
    ConsumerConfiguration as NativeConsumerConfiguration,
    ConsumerCryptoFailureAction as NativeConsumerCryptoFailureAction,
    ConsumerType as NativeConsumerType, InitialPosition as NativeInitialPosition,
};

use crate::{
    error,
    error::Result,
    native::{NativeDrop, NativePointer},
    util,
};

unsafe impl NativeDrop for NativeConsumerConfiguration {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_consumer_configuration_free;
    const TYPE: &'static str = "ConsumerConfiguration";
}

pub struct ConsumerConfiguration {
    inner: NativePointer<NativeConsumerConfiguration>,
}

impl ConsumerConfiguration {
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            let config = pulsar_consumer_configuration_create();
            Self { inner: NativePointer::new_unchecked(config) }
        }
    }

    pub(crate) const fn as_ptr(&self) -> *mut NativeConsumerConfiguration { self.inner.as_ptr() }

    // Schema

    #[must_use]
    pub fn get_consumer_type(&self) -> ConsumerType {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let consumer_type =
            unsafe { pulsar_consumer_configuration_get_consumer_type(self.as_ptr()) };

        consumer_type.into()
    }

    pub fn set_consumer_type(&mut self, consumer_type: ConsumerType) {
        let consumer_type = consumer_type.into();

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_configuration_set_consumer_type(self.as_ptr(), consumer_type);
        }
    }

    // KeySharedPolicy
    // MessageListener
    // ConsumerEventListener

    #[must_use]
    pub fn get_receiver_queue_size(&self) -> u16 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let size = unsafe { pulsar_consumer_configuration_get_receiver_queue_size(self.as_ptr()) };

        size.try_into().expect("FFI: non-negative field")
    }

    pub fn set_receiver_queue_size(&mut self, size: u16) {
        let size = i32::from(size);

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_configuration_set_receiver_queue_size(self.as_ptr(), size);
        }
    }

    #[must_use]
    pub fn get_max_total_receiver_queue_size_across_partitions(&self) -> u16 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let size =
            unsafe { pulsar_consumer_configuration_get_max_pending_chunked_message(self.as_ptr()) };

        size.try_into().expect("FFI: non-negative field")
    }

    pub fn set_max_total_receiver_queue_size_across_partitions(&mut self, size: u16) {
        let size = i32::from(size);

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_configuration_set_max_pending_chunked_message(self.as_ptr(), size);
        }
    }

    #[must_use]
    pub fn get_consumer_name(&self) -> String {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let consumer_name =
            unsafe { CStr::from_ptr(pulsar_consumer_get_consumer_name(self.as_ptr())) };

        consumer_name.to_string_lossy().into_owned()
    }

    /// # Errors
    /// TODO: document
    pub fn set_consumer_name(&mut self, consumer_name: &str) -> Result<()> {
        let consumer_name =
            CString::new(consumer_name).context(error::InvalidCStringSnafu).with_context(|_| {
                error::InvalidConsumerNameSnafu { consumer_name: consumer_name.to_owned() }
            })?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_set_consumer_name(self.as_ptr(), consumer_name.as_ptr());
        }

        Ok(())
    }

    #[must_use]
    pub fn get_unacked_messages_timeout(&self) -> Duration {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let interval = unsafe { pulsar_consumer_get_unacked_messages_timeout_ms(self.as_ptr()) };

        util::convert_duration_from_ffi_c_long_millis(interval).expect("FFI: non-negative field")
    }

    /// # Errors
    /// TODO: document
    pub fn set_unacked_messages_timeout(&mut self, timeout: Duration) -> Result<()> {
        let timeout = util::convert_duration_to_ffi_u64_millis(timeout)?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe { pulsar_consumer_set_unacked_messages_timeout_ms(self.as_ptr(), timeout) };

        Ok(())
    }

    // TickDurationInMs

    #[must_use]
    pub fn get_negative_ack_redelivery_delay(&self) -> Duration {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let millis =
            unsafe { pulsar_configure_get_negative_ack_redelivery_delay_ms(self.as_ptr()) };

        util::convert_duration_from_ffi_c_long_millis(millis).expect("FFI: non-negative field")
    }

    /// # Errors
    /// TODO: document
    pub fn set_negative_ack_redelivery_delay(&mut self, delay: Duration) -> Result<()> {
        let delay = util::convert_duration_to_ffi_c_long_millis(delay)?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe { pulsar_configure_set_negative_ack_redelivery_delay_ms(self.as_ptr(), delay) };

        Ok(())
    }

    #[must_use]
    pub fn get_ack_grouping_time(&self) -> Duration {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let millis = unsafe { pulsar_configure_get_ack_grouping_time_ms(self.as_ptr()) };

        util::convert_duration_from_ffi_c_long_millis(millis).expect("FFI: non-negative field")
    }

    /// # Errors
    /// TODO: document
    pub fn set_ack_grouping_time(&mut self, time: Duration) -> Result<()> {
        let time = util::convert_duration_to_ffi_c_long_millis(time)?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe { pulsar_configure_set_ack_grouping_time_ms(self.as_ptr(), time) };

        Ok(())
    }

    // AckGroupingMaxSize
    // BrokerConsumerStatsCacheTimeInMs
    // EncryptionEnabled
    // CryptoKeyReader
    // CryptoFailureAction

    #[must_use]
    pub fn get_read_compacted(&self) -> bool {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let compacted = unsafe { pulsar_consumer_is_read_compacted(self.as_ptr()) };

        util::convert_bool_from_ffi_c_int(compacted)
    }

    pub fn set_read_compacted(&mut self, compacted: bool) {
        let compacted = util::convert_bool_to_ffi(compacted);

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_set_read_compacted(self.as_ptr(), compacted);
        }
    }

    // PatternAutoDiscoveryPeriod

    #[must_use]
    pub fn get_subscription_initial_position(&self) -> InitialPosition {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let initial_position =
            unsafe { pulsar_consumer_get_subscription_initial_position(self.as_ptr()) };

        match initial_position {
            0 => InitialPosition::Latest,
            1 => InitialPosition::Earliest,
            _ => unreachable!("FFI: non `InitialPosition` enum value"),
        }
    }

    pub fn set_subscription_initial_position(&mut self, initial_position: InitialPosition) {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_consumer_set_subscription_initial_position(
                self.as_ptr(),
                initial_position.into(),
            );
        }
    }

    // BatchReceivePolicy
    // ReplicateSubscriptionStateEnabled
    // PriorityLevel
    // MaxPendingChunkedMessage
    // AutoAckOldestChunkedMessageOnQueueFull
    // ExpireTimeOfIncompleteChunkedMessageMs
    // StartMessageIdInclusive
}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for ConsumerConfiguration {}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ConsumerConfiguration {}

impl Default for ConsumerConfiguration {
    fn default() -> Self { Self::new() }
}

pub enum ConsumerType {
    Exclusive,
    Shared,
    Failover,
    KeyShared,
}

impl From<ConsumerType> for NativeConsumerType {
    fn from(consumer_type: ConsumerType) -> Self {
        match consumer_type {
            ConsumerType::Exclusive => Self::pulsar_ConsumerExclusive,
            ConsumerType::Shared => Self::pulsar_ConsumerShared,
            ConsumerType::Failover => Self::pulsar_ConsumerFailover,
            ConsumerType::KeyShared => Self::pulsar_ConsumerKeyShared,
        }
    }
}

impl From<NativeConsumerType> for ConsumerType {
    fn from(consumer_type: NativeConsumerType) -> Self {
        match consumer_type {
            NativeConsumerType::pulsar_ConsumerExclusive => Self::Exclusive,
            NativeConsumerType::pulsar_ConsumerShared => Self::Shared,
            NativeConsumerType::pulsar_ConsumerFailover => Self::Failover,
            NativeConsumerType::pulsar_ConsumerKeyShared => Self::KeyShared,
        }
    }
}

pub enum InitialPosition {
    Latest,
    Earliest,
}

impl From<InitialPosition> for NativeInitialPosition {
    fn from(initial_position: InitialPosition) -> Self {
        match initial_position {
            InitialPosition::Latest => Self::initial_position_latest,
            InitialPosition::Earliest => Self::initial_position_earliest,
        }
    }
}

impl From<NativeInitialPosition> for InitialPosition {
    fn from(initial_position: NativeInitialPosition) -> Self {
        match initial_position {
            NativeInitialPosition::initial_position_latest => Self::Latest,
            NativeInitialPosition::initial_position_earliest => Self::Earliest,
        }
    }
}

pub enum ConsumerCryptoFailureAction {
    Fail,
    Discard,
    Consume,
}

impl From<ConsumerCryptoFailureAction> for NativeConsumerCryptoFailureAction {
    fn from(crypto_failure_action: ConsumerCryptoFailureAction) -> Self {
        match crypto_failure_action {
            ConsumerCryptoFailureAction::Fail => Self::pulsar_ConsumerFail,
            ConsumerCryptoFailureAction::Discard => Self::pulsar_ConsumerDiscard,
            ConsumerCryptoFailureAction::Consume => Self::pulsar_ConsumerConsume,
        }
    }
}

impl From<NativeConsumerCryptoFailureAction> for ConsumerCryptoFailureAction {
    fn from(crypto_failure_action: NativeConsumerCryptoFailureAction) -> Self {
        match crypto_failure_action {
            NativeConsumerCryptoFailureAction::pulsar_ConsumerFail => Self::Fail,
            NativeConsumerCryptoFailureAction::pulsar_ConsumerDiscard => Self::Discard,
            NativeConsumerCryptoFailureAction::pulsar_ConsumerConsume => Self::Consume,
        }
    }
}
