use std::{
    ffi::{c_int, c_long, c_uint, CStr, CString, OsStr},
    path::Path,
    time::Duration,
};

use snafu::ResultExt;

use crate::{error, error::Result};

pub fn convert_duration_from_ffi_c_int_seconds(duration: i32) -> Result<Duration> {
    let seconds = duration
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: duration.to_string(),
            range: format!("[{},{}]", u64::MIN, u64::MAX),
        })
        .context(error::InvalidNumberSnafu)?;

    Ok(Duration::from_secs(seconds))
}

pub fn convert_duration_from_ffi_c_long_millis(duration: c_long) -> Result<Duration> {
    let millis = duration
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: duration.to_string(),
            range: format!("[{},{}]", u64::MIN, u64::MAX),
        })
        .context(error::InvalidNumberSnafu)?;

    Ok(Duration::from_millis(millis))
}

pub fn convert_duration_to_ffi_c_int_seconds(duration: Duration) -> Result<c_int> {
    let seconds = duration.as_secs();
    let seconds = seconds
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: seconds.to_string(),
            range: format!("[{},{}]", c_int::MIN, c_int::MAX),
        })
        .with_context(|_| error::InvalidDurationSnafu { duration, unit: "seconds" })?;

    Ok(seconds)
}

pub fn convert_duration_to_ffi_c_uint_seconds(duration: Duration) -> Result<c_uint> {
    let seconds = duration.as_secs();
    let seconds = seconds
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: seconds.to_string(),
            range: format!("[{},{}]", c_uint::MIN, c_uint::MAX),
        })
        .with_context(|_| error::InvalidDurationSnafu { duration, unit: "seconds" })?;

    Ok(seconds)
}

pub fn convert_duration_to_ffi_c_long_millis(duration: Duration) -> Result<c_long> {
    let millis = duration.as_millis();
    let millis = millis
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: millis.to_string(),
            range: format!("[{},{}]", c_long::MIN, c_long::MAX),
        })
        .with_context(|_| error::InvalidDurationSnafu { duration, unit: "milliseconds" })?;

    Ok(millis)
}

pub fn convert_duration_to_ffi_u64_millis(duration: Duration) -> Result<u64> {
    let millis = duration.as_millis();
    let millis = millis
        .try_into()
        .with_context(|_| error::NumberOutOfRangeSnafu {
            number: millis.to_string(),
            range: format!("[{},{}]", u64::MIN, u64::MAX),
        })
        .with_context(|_| error::InvalidDurationSnafu { duration, unit: "milliseconds" })?;

    Ok(millis)
}

// ALLOW: non unix system may encounter error
#[allow(clippy::unnecessary_wraps)]
pub fn convert_path_from_ffi_c_str(path: &CStr) -> Result<&Path> {
    #[cfg(unix)]
    let os_str = {
        use std::os::unix::prelude::OsStrExt;
        OsStr::from_bytes(path.to_bytes())
    };

    #[cfg(not(unix))]
    let os_str = OsStr::new(
        path.to_str()
            .context(error::InvalidStringUtf8Snafu)
            .with_context(|_| error::InvalidPathCStrSnafu { c_str: path.to_owned() })?,
    );

    Ok(Path::new(os_str))
}

// NOTE: https://users.rust-lang.org/t/easy-way-to-pass-a-path-to-c/51829
pub fn convert_path_to_ffi_c_str(path: &Path) -> Result<CString> {
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::prelude::OsStrExt;
        path.as_os_str().as_bytes()
    };

    #[cfg(not(unix))]
    let bytes = {
        use snafu::OptionExt;
        path.to_str()
            .context(error::InvalidStringSnafu)
            .with_context(|_| error::InvalidPathSnafu { path: path.to_path_buf() })?
    };

    CString::new(bytes)
        .context(error::InvalidCStringSnafu)
        .with_context(|_| error::InvalidPathSnafu { path: path.to_path_buf() })
}

pub const fn convert_bool_from_ffi_c_int(v: c_int) -> bool { v != 0 }

pub fn convert_bool_to_ffi(v: bool) -> c_int { v.into() }
