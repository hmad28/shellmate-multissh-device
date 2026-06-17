use super::SyncBackend;
use crate::errors::{AppError, AppResult};
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// S3-compatible sync backend.
/// Supports AWS S3, MinIO, Backblaze B2, and other S3-compatible services.
///
/// Credentials JSON format:
/// ```json
/// {
///   "access_key": "AKIA...",
///   "secret_key": "...",
///   "region": "us-east-1",
///   "bucket": "shellmate-sync"
/// }
/// ```
pub struct S3Backend {
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
    prefix: String,
}

#[derive(serde::Deserialize)]
struct S3Credentials {
    access_key: String,
    secret_key: String,
    region: String,
    bucket: String,
}

impl S3Backend {
    pub fn new(endpoint_url: &str, credentials: Option<&str>) -> AppResult<Self> {
        let creds: S3Credentials = credentials
            .ok_or_else(|| AppError::InvalidInput("S3 credentials required".into()))
            .and_then(|c| {
                serde_json::from_str(c)
                    .map_err(|e| AppError::InvalidInput(format!("invalid S3 credentials: {e}")))
            })?;

        let endpoint = endpoint_url.trim_end_matches('/').to_string();
        // Use a fixed prefix to namespace ShellMate objects.
        let prefix = "shellmate/".to_string();

        Ok(Self {
            endpoint,
            bucket: creds.bucket,
            access_key: creds.access_key,
            secret_key: creds.secret_key,
            region: creds.region,
            prefix,
        })
    }

    fn object_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket, key)
    }

    fn list_url(&self) -> String {
        format!(
            "{}/{}?list-type=2&prefix={}",
            self.endpoint, self.bucket, self.prefix
        )
    }

    /// Sign an S3 request using AWS Signature V4.
    fn sign_request(
        &self,
        method: &str,
        url: &str,
        headers: &mut Vec<(String, String)>,
        body: &[u8],
    ) -> AppResult<()> {
        let now = Utc::now();
        let date_stamp = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Parse URL to get host and path.
        let parsed =
            reqwest::Url::parse(url).map_err(|e| AppError::Internal(format!("URL parse: {e}")))?;
        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();
        let query = parsed.query().unwrap_or("");

        headers.push(("x-amz-date".into(), amz_date.clone()));
        headers.push((
            "x-amz-content-sha256".into(),
            hex::encode(Sha256::digest(body)),
        ));
        headers.push(("host".into(), host.to_string()));

        // Canonical request.
        let canonical_headers: String = headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let signed_headers: String = headers
            .iter()
            .map(|(k, _)| k.to_lowercase())
            .collect::<Vec<_>>()
            .join(";");

        let canonical_request = format!(
            "{method}\n{path}\n{query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}",
            payload_hash = hex::encode(Sha256::digest(body))
        );

        // String to sign.
        let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        // Signing key.
        let k_date = hmac_sha256(format!("AWS4{}", self.secret_key).as_bytes(), &date_stamp);
        let k_region = hmac_sha256(&k_date, &self.region);
        let k_service = hmac_sha256(&k_region, "s3");
        let k_signing = hmac_sha256(&k_service, "aws4_request");

        let signature = hex::encode(hmac_sha256(&k_signing, &string_to_sign));

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        headers.push(("authorization".into(), authorization));
        Ok(())
    }
}

fn hmac_sha256(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

#[async_trait]
impl SyncBackend for S3Backend {
    async fn put(&self, object_id: &str, data: &[u8]) -> AppResult<()> {
        let key = format!("{}{}", self.prefix, object_id);
        let url = self.object_url(&key);
        let mut headers = vec![("content-type".into(), "application/octet-stream".into())];
        self.sign_request("PUT", &url, &mut headers, data)?;

        let client = reqwest::Client::new();
        let mut req = client.put(&url).body(data.to_vec());
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 PUT: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!("S3 PUT: {}", resp.status())));
        }
        Ok(())
    }

    async fn get(&self, object_id: &str) -> AppResult<Vec<u8>> {
        let key = format!("{}{}", self.prefix, object_id);
        let url = self.object_url(&key);
        let mut headers = vec![];
        self.sign_request("GET", &url, &mut headers, b"")?;

        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 GET: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!("S3 GET: {}", resp.status())));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| AppError::Internal(format!("S3 GET body: {e}")))
    }

    async fn list(&self) -> AppResult<Vec<String>> {
        let url = self.list_url();
        let mut headers = vec![];
        self.sign_request("GET", &url, &mut headers, b"")?;

        let client = reqwest::Client::new();
        let mut req = client.get(&url);
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 LIST: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!("S3 LIST: {}", resp.status())));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("S3 LIST body: {e}")))?;

        // Parse ListBucketResult XML to extract object keys.
        // Simple regex-free parsing: look for <Key> tags.
        let mut ids = Vec::new();
        let mut remaining = body.as_str();
        while let Some(start) = remaining.find("<Key>") {
            let after_start = &remaining[start + 5..];
            if let Some(end) = after_start.find("</Key>") {
                let key = &after_start[..end];
                // Strip the prefix to get the object ID.
                if let Some(id) = key.strip_prefix(&self.prefix) {
                    ids.push(id.to_string());
                }
                remaining = &after_start[end..];
            } else {
                break;
            }
        }
        Ok(ids)
    }

    async fn delete(&self, object_id: &str) -> AppResult<()> {
        let key = format!("{}{}", self.prefix, object_id);
        let url = self.object_url(&key);
        let mut headers = vec![];
        self.sign_request("DELETE", &url, &mut headers, b"")?;

        let client = reqwest::Client::new();
        let mut req = client.delete(&url);
        for (k, v) in &headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 DELETE: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!("S3 DELETE: {}", resp.status())));
        }
        Ok(())
    }
}
