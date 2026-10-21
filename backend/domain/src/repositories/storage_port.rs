use async_trait::async_trait;
use std::pin::Pin;
use tokio::io::AsyncRead;

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn upload_stream(
        &self,
        key: &str,
        stream: Pin<Box<dyn AsyncRead + Send>>,
        content_length: u64,
        content_type: &str,
    ) -> Result<String, crate::errors::DomainError>;
}
