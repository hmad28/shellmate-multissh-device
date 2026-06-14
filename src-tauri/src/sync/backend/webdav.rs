use crate::errors::AppResult;
use async_trait::async_trait;
use super::SyncBackend;

/// WebDAV sync backend.
/// Supports Nextcloud, ownCloud, and generic WebDAV servers.
///
/// Credentials JSON format:
/// ```json
/// {
///   "username": "user",
///   "password": "pass"
/// }
/// ```
pub struct WebDavBackend {
    base_url: String,
    username: String,
    password: String,
}

impl WebDavBackend {
    pub fn new(endpoint_url: &str, credentials: Option<&str>) -> AppResult<Self> {
        let (username, password) = if let Some(creds) = credentials {
            let parsed: serde_json::Value = serde_json::from_str(creds)
                .map_err(|e| crate::errors::AppError::InvalidInput(format!("invalid credentials: {e}")))?;
            let user = parsed["username"].as_str().unwrap_or("").to_string();
            let pass = parsed["password"].as_str().unwrap_or("").to_string();
            (user, pass)
        } else {
            (String::new(), String::new())
        };

        Ok(Self {
            base_url: endpoint_url.trim_end_matches('/').to_string(),
            username,
            password,
        })
    }

    fn object_url(&self, object_id: &str) -> String {
        format!("{}/{}", self.base_url, object_id)
    }
}

#[async_trait]
impl SyncBackend for WebDavBackend {
    async fn put(&self, object_id: &str, data: &[u8]) -> AppResult<()> {
        let client = reqwest::Client::new();
        let url = self.object_url(object_id);
        let mut req = client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec());
        if !self.username.is_empty() {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await.map_err(|e| crate::errors::AppError::Internal(format!("WebDAV PUT: {e}")))?;
        if !resp.status().is_success() {
            return Err(crate::errors::AppError::Internal(format!("WebDAV PUT: {}", resp.status())));
        }
        Ok(())
    }

    async fn get(&self, object_id: &str) -> AppResult<Vec<u8>> {
        let client = reqwest::Client::new();
        let url = self.object_url(object_id);
        let mut req = client.get(&url);
        if !self.username.is_empty() {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await.map_err(|e| crate::errors::AppError::Internal(format!("WebDAV GET: {e}")))?;
        if !resp.status().is_success() {
            return Err(crate::errors::AppError::Internal(format!("WebDAV GET: {}", resp.status())));
        }
        resp.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| crate::errors::AppError::Internal(format!("WebDAV GET body: {e}")))
    }

    async fn list(&self) -> AppResult<Vec<String>> {
        let client = reqwest::Client::new();
        let url = format!("{}/", self.base_url);
        let mut req = client
            .request("PROPFIND".parse().unwrap(), &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(r#"<?xml version="1.0" encoding="utf-8"?><D:propfind xmlns:D="DAV:"><D:allprop/></D:propfind>"#);
        if !self.username.is_empty() {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await.map_err(|e| crate::errors::AppError::Internal(format!("WebDAV PROPFIND: {e}")))?;
        if !resp.status().is_success() {
            return Err(crate::errors::AppError::Internal(format!("WebDAV PROPFIND: {}", resp.status())));
        }

        let body = resp.text().await.map_err(|e| crate::errors::AppError::Internal(format!("WebDAV body: {e}")))?;

        // Parse D:href from PROPFIND response.
        let mut ids = Vec::new();
        let mut remaining = body.as_str();
        while let Some(start) = remaining.find("<D:href>") {
            let after = &remaining[start + 8..];
            if let Some(end) = after.find("</D:href>") {
                let href = &after[..end];
                // Strip base URL prefix to get object ID.
                if let Some(id) = href.strip_prefix(&self.base_url).and_then(|s| s.strip_prefix('/')) {
                    if !id.is_empty() {
                        ids.push(id.to_string());
                    }
                }
                remaining = &after[end..];
            } else {
                break;
            }
        }
        Ok(ids)
    }

    async fn delete(&self, object_id: &str) -> AppResult<()> {
        let client = reqwest::Client::new();
        let url = self.object_url(object_id);
        let mut req = client.delete(&url);
        if !self.username.is_empty() {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await.map_err(|e| crate::errors::AppError::Internal(format!("WebDAV DELETE: {e}")))?;
        if !resp.status().is_success() {
            return Err(crate::errors::AppError::Internal(format!("WebDAV DELETE: {}", resp.status())));
        }
        Ok(())
    }
}
