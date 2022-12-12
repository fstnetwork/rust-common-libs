use std::{
    fmt,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use tracing::instrument;

/// # Safety
/// TODO: document
pub unsafe trait NativeDrop {
    const TYPE: &'static str;

    const DROP: unsafe extern "C" fn(*mut Self);
}

pub struct NativePointer<T>
where
    T: NativeDrop,
{
    pub ptr: NonNull<T>,
}

impl<T> NativePointer<T>
where
    T: NativeDrop,
{
    // FIXME: remove allow
    #[allow(dead_code)]
    pub fn new(ptr: *mut T) -> Option<Self> { Some(Self { ptr: NonNull::new(ptr)? }) }

    // SAFETY: the caller must guarantee that `ptr` is non-null.
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self { ptr: unsafe { NonNull::new_unchecked(ptr) } }
    }

    pub const fn as_ptr(&self) -> *mut T { self.ptr.as_ptr() }
}

impl<T> Drop for NativePointer<T>
where
    T: NativeDrop,
{
    #[instrument(level = "trace", skip(self), fields(r#type = T::TYPE, pointer = ?self.ptr))]
    fn drop(&mut self) {
        tracing::trace!("Destroying");
        unsafe {
            T::DROP(&mut **self);
        }
        tracing::trace!("Destroyed");
    }
}

impl<T> Deref for NativePointer<T>
where
    T: NativeDrop,
{
    type Target = T;

    fn deref(&self) -> &Self::Target { unsafe { self.ptr.as_ref() } }
}

impl<T> DerefMut for NativePointer<T>
where
    T: NativeDrop,
{
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { self.ptr.as_mut() } }
}

impl<T> fmt::Debug for NativePointer<T>
where
    T: NativeDrop,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.ptr.fmt(f) }
}
