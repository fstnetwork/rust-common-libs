use std::{collections::HashMap, ffi::CStr};

use pulsar_client_sys::{
    pulsar_string_list_free, pulsar_string_list_get, pulsar_string_list_size,
    pulsar_string_map_free, pulsar_string_map_get_key, pulsar_string_map_get_value,
    pulsar_string_map_size, StringList, StringMap,
};

use crate::native::{NativeDrop, NativePointer};

unsafe impl NativeDrop for StringMap {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_string_map_free;
    const TYPE: &'static str = "StringMap";
}

unsafe impl NativeDrop for StringList {
    const DROP: unsafe extern "C" fn(*mut Self) = pulsar_string_list_free;
    const TYPE: &'static str = "StringList";
}

pub fn convert_string_map_from_ffi(
    native_map: &NativePointer<StringMap>,
) -> HashMap<String, String> {
    let len = unsafe { pulsar_string_map_size(native_map.as_ptr()) };

    if len == 0 {
        return HashMap::new();
    }

    (0..len)
        .map(|i| {
            let key = unsafe { CStr::from_ptr(pulsar_string_map_get_key(native_map.as_ptr(), i)) };
            let value =
                unsafe { CStr::from_ptr(pulsar_string_map_get_value(native_map.as_ptr(), i)) };

            (key.to_string_lossy().into_owned(), value.to_string_lossy().into_owned())
        })
        .collect()
}

pub fn convert_string_list_from_ffi(native_list: &NativePointer<StringList>) -> Vec<String> {
    let len = unsafe { pulsar_string_list_size(native_list.as_ptr()) };

    if len == 0 {
        return Vec::new();
    }

    (0..len)
        .map(|i| {
            let item = unsafe { CStr::from_ptr(pulsar_string_list_get(native_list.as_ptr(), i)) };

            item.to_string_lossy().into_owned()
        })
        .collect()
}
