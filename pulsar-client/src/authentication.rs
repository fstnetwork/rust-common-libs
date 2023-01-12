use pulsar_client_sys::{pulsar_authentication_free, Authentication as NativeAuthentication};

use crate::native::NativeDrop;

unsafe impl NativeDrop for NativeAuthentication {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_authentication_free;
    const TYPE: &'static str = "Authentication";
}
