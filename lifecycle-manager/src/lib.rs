mod shutdown_state;
mod signal_watcher;
#[cfg(windows)]
mod windows;
mod worker;

use std::{future::Future, pin::Pin, time::Duration};

use futures::future::BoxFuture;
use tokio::task::JoinHandle;

use self::shutdown_state::ShutdownState;
pub use self::{
    signal_watcher::{Builder as SignalWatcherBuilder, SignalWatcher},
    worker::Worker,
};

pub type ShutdownSignal = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

// developing notes
// this should be refactor to include following features:
// 1. worker can still be added after serve
// 2. worker should known what kind of signal
// 3. worker should be shutdown orderly instead of kill all and wait for
// timeout
pub struct LifecycleManager<E> {
    signal_watcher_builder: SignalWatcherBuilder,
    join_handles: Vec<(String, JoinHandle<Result<(), E>>)>,
}

impl<E> Default for LifecycleManager<E>
where
    E: std::error::Error + Send + 'static,
{
    #[inline]
    fn default() -> Self {
        Self { signal_watcher_builder: SignalWatcher::builder(), join_handles: Vec::new() }
    }
}

impl<E> LifecycleManager<E>
where
    E: std::error::Error + Send + 'static,
{
    #[inline]
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[inline]
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.signal_watcher_builder.with_timeout(timeout);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_custom_shutdown(mut self, shutdown_signal: ShutdownSignal) -> Self {
        self.signal_watcher_builder.with_custom_shutdown(shutdown_signal);
        self
    }

    #[inline]
    #[must_use]
    pub fn add_worker(mut self, worker: impl Worker<Error = E> + 'static) -> Self {
        let worker_name = worker.name().to_string();
        let signal = self.signal_watcher_builder.create_shutdown_signal(&worker_name);
        let fut = worker.serve(signal);
        let join_handle = tokio::spawn(fut);
        self.join_handles.push((worker_name, join_handle));
        self
    }

    #[inline]
    #[must_use]
    pub fn add_worker_fn(
        mut self,
        worker_name: &str,
        worker_fn: impl FnOnce(ShutdownSignal) -> BoxFuture<'static, Result<(), E>>,
    ) -> Self {
        let signal = self.signal_watcher_builder.create_shutdown_signal(worker_name);
        let fut = worker_fn(signal);
        let join_handle = tokio::spawn(fut);
        self.join_handles.push((worker_name.to_string(), join_handle));
        self
    }

    /// # Errors
    ///
    /// TODO: document
    pub async fn serve(self) -> Result<(), E> {
        // TODO: this error should be handle
        let signal_watcher = self.signal_watcher_builder.build().expect("TODO");

        for (worker_name, join_handle) in self.join_handles {
            match join_handle.await {
                Ok(Err(worker_error)) => {
                    tracing::warn!(
                        "Error occurs while worker {worker_name} is joined, error: {worker_error}"
                    );
                }
                Ok(_) => (),
                Err(join_error) => {
                    tracing::warn!(
                        "Error occurs while trying to join worker {worker_name}, error: \
                         {join_error}"
                    );
                }
            }
        }

        signal_watcher.wait();
        tracing::info!("All workers are gracefully shutdown!");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // SAFETY: allow: prost
    #![allow(
        unreachable_pub,
        clippy::large_enum_variant,
        clippy::missing_errors_doc,
        clippy::must_use_candidate,
        clippy::return_self_not_must_use,
        clippy::similar_names,
        clippy::too_many_lines,
        clippy::use_self,
        clippy::wildcard_imports
    )]

    use std::{
        net::{Ipv4Addr, SocketAddr},
        time::Duration,
    };

    use async_trait::async_trait;
    use portpicker::pick_unused_port;
    use snafu::Snafu;

    use super::{LifecycleManager, ShutdownSignal, Worker};

    #[derive(Debug, Snafu)]
    enum Error {
        Dummy,
    }

    struct DummyWorker {
        num: u32,
        name: String,
    }

    impl DummyWorker {
        fn new(num: u32) -> Self { Self { num, name: format!("DummyWorker-{}", num) } }
    }

    #[async_trait]
    impl Worker for DummyWorker {
        type Error = Error;

        fn name(&self) -> &str { &self.name }

        async fn serve(self, shutdown_signal: ShutdownSignal) -> Result<(), Self::Error> {
            println!("DummyWorker {} is waiting for shutdown signal", self.num);

            shutdown_signal.await;
            println!("DummyWorker {} is shutting down gracefully", self.num);
            Ok(())
        }
    }

    #[derive(Default)]
    struct AxumServer;

    #[async_trait]
    impl Worker for AxumServer {
        type Error = Error;

        fn name(&self) -> &str { "axum-server" }

        async fn serve(mut self, shutdown_signal: ShutdownSignal) -> Result<(), Self::Error> {
            let port = pick_unused_port().unwrap();

            // SAFETY: allow: use for testing
            #[allow(clippy::unused_async)]
            async fn hello() -> &'static str { "Hello world!" }

            let router = axum::Router::new().route("/hello", axum::routing::get(hello));

            let server = axum::Server::bind(&SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port))
                .serve(router.into_make_service())
                .with_graceful_shutdown(shutdown_signal);

            if let Err(err) = server.await {
                eprintln!("Error occurs while awaiting for AxumServer {err}");
            }

            println!("AxumServer is gracefully shutdown");

            Ok(())
        }
    }

    fn spawn_killer_task() -> ShutdownSignal {
        Box::pin(async {
            let timeout = Duration::from_secs(2);
            println!("Killer task: sleep for {} milliseconds", timeout.as_millis());
            tokio::time::sleep(timeout).await;
            println!("Killer task: send shutdown signal");
        })
    }

    #[cfg(unix)]
    fn spawn_killer_thread() {
        let pid = std::process::id();
        let sig = libc::SIGTERM;

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));

            println!("Send signal {sig} to current process (pid: {pid})");

            unsafe {
                let pid = libc::pid_t::try_from(pid).expect("pid not overflow; qed");
                libc::kill(pid, sig);
            }
        });
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_empty() -> Result<(), Error> {
        let shutdown_signal = spawn_killer_task();
        LifecycleManager::<Error>::new().with_custom_shutdown(shutdown_signal).serve().await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_with_dummy_workers() -> Result<(), Error> {
        let shutdown_signal = spawn_killer_task();
        LifecycleManager::new()
            .with_custom_shutdown(shutdown_signal)
            .add_worker(DummyWorker::new(0))
            .add_worker(DummyWorker::new(1))
            .add_worker(DummyWorker::new(2))
            .add_worker(DummyWorker::new(3))
            .add_worker_fn("worker-function", |shutdown_signal| {
                Box::pin(async move {
                    println!("worker-function: Wait for shutdown signal");
                    shutdown_signal.await;
                    Ok(())
                })
            })
            .serve()
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_with_axum_server() -> Result<(), Error> {
        let shutdown_signal = spawn_killer_task();

        LifecycleManager::new()
            .with_custom_shutdown(shutdown_signal)
            .add_worker(AxumServer::default())
            .serve()
            .await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_with_axum_server_and_dummy_workers() -> Result<(), Error> {
        let shutdown_signal = spawn_killer_task();

        LifecycleManager::new()
            .with_custom_shutdown(shutdown_signal)
            .add_worker(AxumServer::default())
            .add_worker(DummyWorker::new(1))
            .serve()
            .await?;

        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test]
    #[cfg_attr(miri, ignore)]
    async fn test_with_axum_server_and_dummy_workers_with_unix_signal() -> Result<(), Error> {
        spawn_killer_thread();

        LifecycleManager::<Error>::new()
            .add_worker(AxumServer::default())
            .add_worker(DummyWorker::new(0))
            .serve()
            .await?;

        Ok(())
    }
}
