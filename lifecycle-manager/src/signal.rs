use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::stream::Stream;
use tokio::signal::unix::Signal;

pub trait SignalCompatExt {
    fn compat(self) -> SignalCompat;
}

impl SignalCompatExt for Signal {
    fn compat(self) -> SignalCompat { SignalCompat::new(self) }
}

pub struct SignalCompat {
    inner: Signal,
}

impl SignalCompat {
    const fn new(signal: Signal) -> Self { Self { inner: signal } }
}

impl Unpin for SignalCompat {}

impl Stream for SignalCompat {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}
