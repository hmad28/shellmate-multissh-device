use super::SyncBackend;
use crate::errors::{AppError, AppResult};
use async_trait::async_trait;

/// Self-hosted HTTP sync backend.
/// Expects a simple REST API:
///   PUT  /objects/{id}  — upload
///   GET  /objects/{id}  — download
///   GET  /objects       — list (returns JSON array of IDs)
///   DELETE /objects/{id} — delete
pub struct HttpBackend {
    base_url: String,
    auth_header: Option<String>,
}

impl HttpBackend {
    pub fn new(endpoint_url: &str, credentials: Option<&str>) -> AppResult<Self> {
        let base_url = endpoint_url.trim_end_matches('/').to_string();
        let auth_header = credentials.map(|c| format!("Bearer {c}"));
        Ok(Self {
            base_url,
            auth_header,
        })
    }

    fn client(&self) -> AppResult<reqwest::Client> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP client: {e}")))
    }
}

#[async_trait]
impl SyncBackend for HttpBackend {
    async fn put(&self, object_id: &str, data: &[u8]) -> AppResult<()> {
        let client = self.client()?;
        let url = format!("{}/objects/{}", self.base_url, object_id);
        let mut req = client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec());
        if let Some(ref auth) = self.auth_header {
            req = req.header("Authorization", auth);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP PUT: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "HTTP PUT failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn get(&self, object_id: &str) -> AppResult<Vec<u8>> {
        let client = self.client()?;
        let url = format!("{}/objects/{}", self.base_url, object_id);
        let mut req = client.get(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.header("Authorization", auth);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP GET: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "HTTP GET failed: {}",
                resp.status()
            )));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| AppError::Internal(format!("HTTP GET body: {e}")))
    }

    async fn list(&self) -> AppResult<Vec<String>> {
        let client = self.client()?;
        let url = format!("{}/objects", self.base_url);
        let mut req = client.get(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.header("Authorization", auth);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP LIST: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "HTTP LIST failed: {}",
                resp.status()
            )));
        }
        let ids: Vec<String> = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP LIST parse: {e}")))?;
        Ok(ids)
    }

    async fn delete(&self, object_id: &str) -> AppResult<()> {
        let client = self.client()?;
        let url = format!("{}/objects/{}", self.base_url, object_id);
        let mut req = client.delete(&url);
        if let Some(ref auth) = self.auth_header {
            req = req.header("Authorization", auth);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP DELETE: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "HTTP DELETE failed: {}",
                resp.status()
            )));
        }
        Ok(())
    }
}
