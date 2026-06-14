use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory object store. In production, replace with persistent storage.
type ObjectStore = Arc<RwLock<HashMap<String, StoredObject>>>;

#[derive(Clone)]
struct StoredObject {
    data: Vec<u8>,
    content_hash: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ObjectInfo {
    id: String,
    size: usize,
    content_hash: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    objects: usize,
}

#[derive(Deserialize)]
struct PutRequest {
    data: Vec<u8>,
}

/// GET /health — health check.
async fn health(State(store): State<ObjectStore>) -> Json<HealthResponse> {
    let objects = store.read().await.len();
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        objects,
    })
}

/// PUT /objects/:id — upload an object.
async fn put_object(
    Path(id): Path<String>,
    State(store): State<ObjectStore>,
    body: Vec<u8>,
) -> Result<Json<ObjectInfo>, StatusCode> {
    let now = chrono_like_now();
    let hash = hex::encode(Sha256::digest(&body));
    let size = body.len();

    let obj = StoredObject {
        data: body,
        content_hash: hash.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let mut store = store.write().await;
    store.insert(id.clone(), obj);

    Ok(Json(ObjectInfo {
        id,
        size,
        content_hash: hash,
        created_at: now.clone(),
        updated_at: now,
    }))
}

/// GET /objects/:id — download an object.
async fn get_object(
    Path(id): Path<String>,
    State(store): State<ObjectStore>,
) -> Result<Vec<u8>, StatusCode> {
    let store = store.read().await;
    store
        .get(&id)
        .map(|obj| obj.data.clone())
        .ok_or(StatusCode::NOT_FOUND)
}

/// GET /objects — list all object IDs.
async fn list_objects(State(store): State<ObjectStore>) -> Json<Vec<String>> {
    let store = store.read().await;
    let ids: Vec<String> = store.keys().cloned().collect();
    Json(ids)
}

/// DELETE /objects/:id — delete an object.
async fn delete_object(
    Path(id): Path<String>,
    State(store): State<ObjectStore>,
) -> Result<StatusCode, StatusCode> {
    let mut store = store.write().await;
    store.remove(&id).map(|_| StatusCode::OK).ok_or(StatusCode::NOT_FOUND)
}

/// Simple timestamp without chrono dependency.
fn chrono_like_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", now.as_secs(), now.subsec_millis())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let store: ObjectStore = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/objects", get(list_objects))
        .route("/objects/:id", put(put_object).get(get_object).delete(delete_object))
        .with_state(store)
        .layer(tower_http::cors::CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    log::info!("ShellMate sync server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
