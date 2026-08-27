use async_trait::async_trait;
use domain::repositories::StoragePort;
use domain::errors::DomainError;
use std::pin::Pin;
use tokio::io::AsyncRead;

pub struct S3Adapter;

#[async_trait]
impl StoragePort for S3Adapter {
    async fn upload_stream(
        &self,
        key: &str,
        _stream: Pin<Box<dyn AsyncRead + Send>>,
        _content_length: u64,
        _content_type: &str,
    ) -> Result<String, DomainError> {
        // Return a mock S3 CDN URI
        Ok(format!("https://cdn.has.ltd/uploads/{}", key))
    }
}
