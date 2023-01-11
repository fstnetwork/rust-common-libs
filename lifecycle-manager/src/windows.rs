// reference: https://github.com/tokio-rs/tokio/blob/tokio-stream-0.1.11/tokio-stream/src/wrappers/signal_windows.rs

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use tokio::signal::windows::{CtrlClose, CtrlShutdown};

/// A wrapper around [`CtrlClose`] that implements [`Stream`].
///
/// [`CtrlClose`]: struct@tokio::signal::windows::CtrlClose
/// [`Stream`]: trait@crate::Stream
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(all(windows, feature = "signal"))))]
pub struct CtrlCloseStream {
    inner: CtrlClose,
}

impl CtrlCloseStream {
    /// Create a new `CtrlCloseStream`.
    pub fn new(interval: CtrlClose) -> Self { Self { inner: interval } }

    /// Get back the inner `CtrlClose`.
    #[allow(unused)]
    pub fn into_inner(self) -> CtrlClose { self.inner }
}

impl Stream for CtrlCloseStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        self.inner.poll_recv(cx)
    }
}

impl AsRef<CtrlClose> for CtrlCloseStream {
    fn as_ref(&self) -> &CtrlClose { &self.inner }
}

impl AsMut<CtrlClose> for CtrlCloseStream {
    fn as_mut(&mut self) -> &mut CtrlClose { &mut self.inner }
}

/// A wrapper around [`CtrlShutdown`] that implements [`Stream`].
///
/// [`CtrlShutdown`]: struct@tokio::signal::windows::CtrlShutdown
/// [`Stream`]: trait@crate::Stream
#[derive(Debug)]
#[cfg_attr(docsrs, doc(cfg(all(windows, feature = "signal"))))]
pub struct CtrlShutdownStream {
    inner: CtrlShutdown,
}

impl CtrlShutdownStream {
    /// Create a new `CtrlShutdownStream`.
    pub fn new(interval: CtrlShutdown) -> Self { Self { inner: interval } }

    /// Get back the inner `CtrlShutdown`.
    #[allow(unused)]
    pub fn into_inner(self) -> CtrlShutdown { self.inner }
}

impl Stream for CtrlShutdownStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        self.inner.poll_recv(cx)
    }
}

impl AsRef<CtrlShutdown> for CtrlShutdownStream {
    fn as_ref(&self) -> &CtrlShutdown { &self.inner }
}

impl AsMut<CtrlShutdown> for CtrlShutdownStream {
    fn as_mut(&mut self) -> &mut CtrlShutdown { &mut self.inner }
}
