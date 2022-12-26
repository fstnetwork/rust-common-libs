use std::ffi::{CStr, CString};

use pulsar_client_sys::{
    pulsar_producer_configuration_create, pulsar_producer_configuration_free,
    pulsar_producer_configuration_get_producer_name,
    pulsar_producer_configuration_set_producer_name,
    ProducerConfiguration as NativeProducerConfiguration,
};
use snafu::ResultExt;

use crate::{
    error,
    error::Result,
    native::{NativeDrop, NativePointer},
};

unsafe impl NativeDrop for NativeProducerConfiguration {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_producer_configuration_free;
    const TYPE: &'static str = "ProducerConfiguration";
}

pub struct ProducerConfiguration {
    inner: NativePointer<NativeProducerConfiguration>,
}

impl ProducerConfiguration {
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            let config = pulsar_producer_configuration_create();
            Self { inner: NativePointer::new_unchecked(config) }
        }
    }

    pub(crate) const fn as_ptr(&self) -> *mut NativeProducerConfiguration { self.inner.as_ptr() }

    #[must_use]
    pub fn get_producer_name(&self) -> String {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let producer_name = unsafe {
            CStr::from_ptr(pulsar_producer_configuration_get_producer_name(self.as_ptr()))
        };

        producer_name.to_string_lossy().into_owned()
    }

    /// # Errors
    /// TODO: document
    pub fn set_producer_name(&mut self, producer_name: &str) -> Result<()> {
        let producer_name =
            CString::new(producer_name).context(error::InvalidCStringSnafu).with_context(|_| {
                error::InvalidProducerNameSnafu { producer_name: producer_name.to_owned() }
            })?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_producer_configuration_set_producer_name(self.as_ptr(), producer_name.as_ptr());
        }

        Ok(())
    }
}

impl Default for ProducerConfiguration {
    fn default() -> Self { Self::new() }
}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for ProducerConfiguration {}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ProducerConfiguration {}
