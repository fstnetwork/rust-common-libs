use std::{ffi::CStr, path::Path, ptr, time::Duration};

use pulsar_client_sys::{
    pulsar_client_configuration_create, pulsar_client_configuration_free,
    pulsar_client_configuration_get_concurrent_lookup_request,
    pulsar_client_configuration_get_io_threads, pulsar_client_configuration_get_memory_limit,
    pulsar_client_configuration_get_message_listener_threads,
    pulsar_client_configuration_get_operation_timeout_seconds,
    pulsar_client_configuration_get_stats_interval_in_seconds,
    pulsar_client_configuration_get_tls_trust_certs_file_path,
    pulsar_client_configuration_is_tls_allow_insecure_connection,
    pulsar_client_configuration_is_use_tls, pulsar_client_configuration_is_validate_hostname,
    pulsar_client_configuration_set_concurrent_lookup_request,
    pulsar_client_configuration_set_io_threads, pulsar_client_configuration_set_logger,
    pulsar_client_configuration_set_memory_limit,
    pulsar_client_configuration_set_message_listener_threads,
    pulsar_client_configuration_set_operation_timeout_seconds,
    pulsar_client_configuration_set_stats_interval_in_seconds,
    pulsar_client_configuration_set_tls_allow_insecure_connection,
    pulsar_client_configuration_set_tls_trust_certs_file_path,
    pulsar_client_configuration_set_use_tls, pulsar_client_configuration_set_validate_hostname,
    ClientConfiguration as NativeClientConfiguration,
};

use crate::{
    client,
    error::Result,
    native::{NativeDrop, NativePointer},
    utils,
};

unsafe impl NativeDrop for NativeClientConfiguration {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_client_configuration_free;
    const TYPE: &'static str = "ClientConfiguration";
}

#[derive(Debug)]
pub struct ClientConfiguration {
    inner: NativePointer<NativeClientConfiguration>,
}

impl ClientConfiguration {
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            let config = pulsar_client_configuration_create();
            pulsar_client_configuration_set_logger(config, Some(client::log), ptr::null_mut());
            Self { inner: NativePointer::new_unchecked(config) }
        }
    }

    pub(crate) const fn as_ptr(&self) -> *mut NativeClientConfiguration { self.inner.as_ptr() }

    /// Returns the client memory limit in bytes
    #[must_use]
    pub fn get_memory_limit(&self) -> u64 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe { pulsar_client_configuration_get_memory_limit(self.as_ptr()) }
    }

    /// Configure a limit on the amount of memory that will be allocated by this
    /// client instance. Setting this to 0 will disable the limit. By
    /// default this is disabled.
    ///
    /// # Arguments
    /// * `memory_limit_bytes` - the memory limit
    pub fn set_memory_limit(&mut self, memory_limit_bytes: u64) {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_memory_limit(self.as_ptr(), memory_limit_bytes);
        }
    }

    // TODO: auth

    /// Returns the client operations timeout in seconds
    #[must_use]
    pub fn get_operation_timeout(&self) -> Duration {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let timeout =
            unsafe { pulsar_client_configuration_get_operation_timeout_seconds(self.as_ptr()) };

        utils::convert_duration_from_ffi_c_int_seconds(timeout).expect("FFI: non-negative field")
    }

    /// Set timeout on client operations (subscribe, create producer, close,
    /// unsubscribe) Default is 30 seconds.
    ///
    /// # Arguments
    /// * `timeout` - the timeout after which the operation will be considered
    ///   as failed
    /// # Errors
    /// TODO: document
    pub fn set_operation_timeout(&mut self, timeout: Duration) -> Result<()> {
        let timeout = utils::convert_duration_to_ffi_c_int_seconds(timeout)?;
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_operation_timeout_seconds(self.as_ptr(), timeout);
        }

        Ok(())
    }

    #[must_use]
    pub fn get_io_threads(&self) -> u16 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let threads = unsafe { pulsar_client_configuration_get_io_threads(self.as_ptr()) };

        threads.try_into().expect("FFI: non-negative field")
    }

    pub fn set_io_threads(&mut self, threads: u16) {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_io_threads(self.as_ptr(), threads.into());
        }
    }

    #[must_use]
    pub fn get_message_listener_threads(&self) -> u16 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let threads =
            unsafe { pulsar_client_configuration_get_message_listener_threads(self.as_ptr()) };

        threads.try_into().expect("FFI: non-negative field")
    }

    pub fn set_message_listener_threads(&mut self, threads: u16) {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_message_listener_threads(self.as_ptr(), threads.into());
        }
    }

    #[must_use]
    pub fn get_message_get_concurrent_lookup_request(&self) -> u16 {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let concurrency =
            unsafe { pulsar_client_configuration_get_concurrent_lookup_request(self.as_ptr()) };

        concurrency.try_into().expect("FFI: non-negative field")
    }

    pub fn set_message_get_concurrent_lookup_request(&mut self, concurrent_lookup_request: u16) {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_concurrent_lookup_request(
                self.as_ptr(),
                concurrent_lookup_request.into(),
            );
        }
    }

    // TODO: logger

    #[must_use]
    pub fn is_use_tls(&self) -> bool {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let use_tls = unsafe { pulsar_client_configuration_is_use_tls(self.as_ptr()) };

        utils::convert_bool_from_ffi_c_int(use_tls)
    }

    pub fn set_use_tls(&mut self, use_tls: bool) {
        let use_tls = utils::convert_bool_to_ffi(use_tls);
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_use_tls(self.as_ptr(), use_tls);
        }
    }

    // pub fn get_tls_private_key_file_path(&self) -> Result<&Path> {
    // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
    // let path = unsafe {
    // CStr::from_ptr(pulsar_client_configuration_get_tls_private_key_file_path(
    // self.inner.as_ptr(),
    // ))
    // };
    //
    // utils::convert_from_ffi_path(path)
    // }
    //
    // pub fn set_tls_private_key_file_path<P>(&mut self, tls_key_file_path: &P) ->
    // Result<()> where
    // P: AsRef<Path> + ?Sized,
    // {
    // let path = utils::convert_to_ffi_path(tls_key_file_path.as_ref())?;
    // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
    // unsafe {
    // pulsar_client_configuration_set_tls_private_key_file_path(
    // &mut *self.inner,
    // path.as_ptr(),
    // );
    // }
    //
    // Ok(())
    // }
    //
    // pub fn get_tls_certificate_file_path(&self) -> Result<&Path> {
    // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
    // let path = unsafe {
    // CStr::from_ptr(pulsar_client_configuration_get_tls_certificate_file_path(
    // self.inner.as_ptr(),
    // ))
    // };
    //
    // utils::convert_from_ffi_path(path)
    // }
    //
    // pub fn set_tls_certificate_file_path<P>(&mut self, tls_key_file_path: &P) ->
    // Result<()> where
    // P: AsRef<Path> + ?Sized,
    // {
    // let path = utils::convert_to_ffi_path(tls_key_file_path.as_ref())?;
    // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
    // unsafe {
    // pulsar_client_configuration_set_tls_certificate_file_path(
    // &mut *self.inner,
    // path.as_ptr(),
    // );
    // }
    //
    // Ok(())
    // }

    /// # Errors
    /// TODO: document
    pub fn get_tls_trust_certs_file_path(&self) -> Result<&Path> {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let path = unsafe {
            CStr::from_ptr(pulsar_client_configuration_get_tls_trust_certs_file_path(self.as_ptr()))
        };

        utils::convert_path_from_ffi_c_str(path)
    }

    /// # Errors
    /// TODO: document
    pub fn set_tls_trust_certs_file_path<P>(&mut self, tls_key_file_path: &P) -> Result<()>
    where
        P: AsRef<Path> + ?Sized,
    {
        let path = utils::convert_path_to_ffi_c_str(tls_key_file_path.as_ref())?;
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_tls_trust_certs_file_path(self.as_ptr(), path.as_ptr());
        }

        Ok(())
    }

    #[must_use]
    pub fn is_tls_allow_insecure_connection(&self) -> bool {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let allow_insecure =
            unsafe { pulsar_client_configuration_is_tls_allow_insecure_connection(self.as_ptr()) };

        utils::convert_bool_from_ffi_c_int(allow_insecure)
    }

    pub fn set_tls_allow_insecure_connection(&mut self, allow_insecure: bool) {
        let allow_insecure = utils::convert_bool_to_ffi(allow_insecure);
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_tls_allow_insecure_connection(
                self.as_ptr(),
                allow_insecure,
            );
        }
    }

    #[must_use]
    pub fn is_validate_hostname(&self) -> bool {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let validate_hostname =
            unsafe { pulsar_client_configuration_is_validate_hostname(self.as_ptr()) };

        utils::convert_bool_from_ffi_c_int(validate_hostname)
    }

    pub fn set_validate_hostname(&mut self, validate_hostname: bool) {
        let validate_hostname = utils::convert_bool_to_ffi(validate_hostname);
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_validate_hostname(self.as_ptr(), validate_hostname);
        }
    }

    // ListenerName

    #[must_use]
    pub fn get_stats_interval(&self) -> Duration {
        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        let interval =
            unsafe { pulsar_client_configuration_get_stats_interval_in_seconds(self.as_ptr()) };

        Duration::from_secs(interval.into())
    }

    /// # Errors
    /// TODO: document
    pub fn set_stats_interval(&mut self, interval: Duration) -> Result<()> {
        let interval = utils::convert_duration_to_ffi_c_uint_seconds(interval)?;

        // SAFETY: FFI ‒ pointers are valid, it doesn't take ownership; qed
        unsafe {
            pulsar_client_configuration_set_stats_interval_in_seconds(self.as_ptr(), interval);
        }

        Ok(())
    }

    // PartititionsUpdateInterval
    // ConnectionTimeout
}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for ClientConfiguration {}

// SAFETY: the does referenced is not aliased
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ClientConfiguration {}

impl Default for ClientConfiguration {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ClientConfiguration;

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test() {
        let mut config = ClientConfiguration::new();

        config.set_tls_trust_certs_file_path("/doge").unwrap();

        assert_eq!(config.get_tls_trust_certs_file_path().unwrap(), Path::new("/doge"));
    }
}
