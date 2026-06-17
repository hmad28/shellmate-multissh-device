pub mod http;
pub mod s3;
pub mod webdav;

use crate::errors::{AppError, AppResult};
use async_trait::async_trait;

/// Trait for sync storage backends.
#[async_trait]
pub trait SyncBackend: Send + Sync {
    async fn put(&self, object_id: &str, data: &[u8]) -> AppResult<()>;
    async fn get(&self, object_id: &str) -> AppResult<Vec<u8>>;
    async fn list(&self) -> AppResult<Vec<String>>;
    async fn delete(&self, object_id: &str) -> AppResult<()>;
}

/// Create a backend instance from configuration.
pub fn create_backend(
    backend_type: &str,
    endpoint_url: &str,
    credentials: Option<&str>,
) -> AppResult<Box<dyn SyncBackend>> {
    match backend_type {
        "http" => Ok(Box::new(http::HttpBackend::new(endpoint_url, credentials)?)),
        "s3" => Ok(Box::new(s3::S3Backend::new(endpoint_url, credentials)?)),
        "webdav" => Ok(Box::new(webdav::WebDavBackend::new(
            endpoint_url,
            credentials,
        )?)),
        _ => Err(AppError::InvalidInput(format!(
            "unsupported sync backend: {backend_type}"
        ))),
    }
}
