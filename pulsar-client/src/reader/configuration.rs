use pulsar_client_sys::{
    pulsar_reader_configuration_free, ReaderConfiguration as NativeReaderConfiguration,
};

use crate::native::NativeDrop;

unsafe impl NativeDrop for NativeReaderConfiguration {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_reader_configuration_free;
    const TYPE: &'static str = "ReaderConfiguration";
}
