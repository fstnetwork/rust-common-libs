use pulsar_client_sys::{
    pulsar_producer_configuration_free, ProducerConfiguration as NativeProducerConfiguration,
};

use crate::native::NativeDrop;

unsafe impl NativeDrop for NativeProducerConfiguration {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_producer_configuration_free;
    const TYPE: &'static str = "ProducerConfiguration";
}
