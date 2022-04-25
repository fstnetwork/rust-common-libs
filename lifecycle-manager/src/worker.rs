use async_trait::async_trait;

use crate::ShutdownSignal;

#[async_trait]
pub trait Worker {
    type Error;

    fn name(&self) -> &str;

    async fn serve(self, shutdown_signal: ShutdownSignal) -> Result<(), Self::Error>;
}
