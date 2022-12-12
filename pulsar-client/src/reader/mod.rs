mod configuration;

use pulsar_client_sys::{pulsar_reader_free, Reader as NativeReader};

use crate::native::NativeDrop;

unsafe impl NativeDrop for NativeReader {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_reader_free;
    const TYPE: &'static str = "Reader";
}
