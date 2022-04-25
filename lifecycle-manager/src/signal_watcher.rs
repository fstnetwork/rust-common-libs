use std::{io, time::Duration};

use futures::{
    future::FutureExt,
    stream,
    stream::{BoxStream, StreamExt},
};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
    task::JoinHandle,
};

use crate::{ShutdownSignal, ShutdownState, SignalCompatExt};

#[derive(Debug)]
pub struct SignalWatcher {
    join_handle: Option<JoinHandle<()>>,
}

impl SignalWatcher {
    #[inline]
    #[must_use]
    pub fn builder() -> Builder {
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        Builder { shutdown_tx, shutdown_rx, shutdown_signal: None, timeout: None }
    }

    #[inline]
    pub fn wait(self) {
        tracing::info!("Stop SignalWatcher");

        if let Some(join_handle) = self.join_handle {
            join_handle.abort();
        }

        tracing::info!("SignalWatcher is stopped");
    }
}

pub struct Builder {
    shutdown_tx: watch::Sender<()>,
    shutdown_rx: watch::Receiver<()>,
    shutdown_signal: Option<ShutdownSignal>,
    timeout: Option<Duration>,
}

impl Builder {
    #[inline]
    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    #[inline]
    pub fn with_custom_shutdown(&mut self, shutdown_signal: ShutdownSignal) -> &mut Self {
        self.shutdown_signal = Some(shutdown_signal);
        self
    }

    #[must_use]
    pub fn create_shutdown_signal(&self, name: &str) -> ShutdownSignal {
        let name = name.to_string();
        let mut shutdown_rx = self.shutdown_rx.clone();
        let fut = async move {
            match shutdown_rx.changed().await {
                Ok(_) => {
                    tracing::info!("Shutdown signal received, try to shutdown worker `{}`", name);
                }
                Err(_) => {
                    tracing::info!(
                        "Shutdown signal sender is dropped, try to shutdown worker `{}`",
                        name
                    );
                }
            }
        };

        Box::pin(fut)
    }

    /// # Errors
    ///
    /// If [`tokio::signal::unix::signal`
    /// error](fn@tokio::signal::unix::signal#errors).
    pub fn build(self) -> io::Result<SignalWatcher> {
        let (shutdown_tx, internal_shutdown_signal, shutdown_timeout) = {
            (
                self.shutdown_tx,
                self.shutdown_signal,
                self.timeout.unwrap_or_else(|| Duration::from_secs(10)),
            )
        };

        let mut signal_stream = {
            type Signal = BoxStream<'static, ()>;

            let mut streams: Vec<Signal> = vec![
                signal(SignalKind::terminate())?.compat().boxed(),
                signal(SignalKind::interrupt())?.compat().boxed(),
            ];

            if let Some(shutdown_signal) = internal_shutdown_signal {
                streams.push(shutdown_signal.into_stream().boxed());
            }

            stream::select_all(streams)
        };

        let join_handle = tokio::spawn(async move {
            let mut state = ShutdownState::default();
            tracing::info!("SignalWorker is waiting for signals");

            while signal_stream.next().await.is_some() {
                match state.next() {
                    None | Some(ShutdownState::Initial) => unreachable!(),
                    Some(ShutdownState::WaitForSignal) => {
                        tracing::info!("Send shutdown signal to all workers");

                        if let Err(_err) = shutdown_tx.send(()) {
                            tracing::warn!("Failed to send shutdown signal");
                        }
                    }
                    Some(ShutdownState::ShuttingDown) => {
                        tracing::warn!(
                            "Another shutdown signal is received, force exit in {} milliseconds",
                            shutdown_timeout.as_millis()
                        );

                        tokio::spawn(async move {
                            tokio::time::sleep(shutdown_timeout).await;
                            tracing::warn!("Force exit this process");
                            std::process::exit(1);
                        });
                    }
                    Some(ShutdownState::Aborting) => {
                        tracing::error!(
                            "Could not shut down this process gracefully, abort this process"
                        );
                        std::process::abort();
                    }
                }
            }
        });

        Ok(SignalWatcher { join_handle: Some(join_handle) })
    }
}
