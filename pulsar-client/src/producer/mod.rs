mod configuration;

use pulsar_client_sys::{pulsar_producer_free, Producer as NativeProducer};

use crate::native::NativeDrop;

unsafe impl NativeDrop for NativeProducer {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_producer_free;
    const TYPE: &'static str = "Producer";
}
