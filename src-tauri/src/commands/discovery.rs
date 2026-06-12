use crate::errors::AppResult;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Emitter};
use tokio::sync::Mutex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredHost {
    pub hostname: String,
    pub ip_addresses: Vec<String>,
    pub port: u16,
}

struct DiscoveryState {
    daemon: Option<ServiceDaemon>,
    is_broadcasting: bool,
}

impl DiscoveryState {
    fn new() -> Self {
        Self { daemon: None, is_broadcasting: false }
    }
}

pub fn init(app: &mut tauri::App) {
    app.manage(Arc::new(Mutex::new(DiscoveryState::new())));
}

#[tauri::command]
pub async fn start_discovery(app: AppHandle) -> AppResult<()> {
    let state = app.state::<Arc<Mutex<DiscoveryState>>>();
    let mut st = state.lock().await;

    if st.daemon.is_some() {
        return Ok(());
    }

    let mdns = ServiceDaemon::new().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    st.daemon = Some(mdns.clone());

    let service_type = "_ssh._tcp.local.";
    let receiver = mdns.browse(service_type).map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                let host = DiscoveredHost {
                    hostname: info.get_hostname().trim_end_matches(".local.").to_string(),
                    ip_addresses: info.get_addresses().iter().map(|ip| ip.to_string()).collect(),
                    port: info.get_port(),
                };
                let _ = app_clone.emit("discovery:host_found", host);
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_discovery(app: AppHandle) -> AppResult<()> {
    let state = app.state::<Arc<Mutex<DiscoveryState>>>();
    let mut st = state.lock().await;
    
    if let Some(daemon) = st.daemon.take() {
        let _ = daemon.shutdown();
    }
    Ok(())
}

#[tauri::command]
pub async fn start_broadcasting(app: AppHandle) -> AppResult<()> {
    use mdns_sd::ServiceInfo;
    let state = app.state::<Arc<Mutex<DiscoveryState>>>();
    let mut st = state.lock().await;

    if st.is_broadcasting {
        return Ok(());
    }

    let mdns = if let Some(daemon) = &st.daemon {
        daemon.clone()
    } else {
        let daemon = ServiceDaemon::new().map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
        st.daemon = Some(daemon.clone());
        daemon
    };

    let host_name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "shellmate".to_string());

    let instance_name = format!("Shellmate-{}", host_name);
    let service_type = "_ssh._tcp.local.";
    let host_name_fqdn = format!("{}.local.", host_name);
    
    let service_info = ServiceInfo::new(
        service_type,
        &instance_name,
        &host_name_fqdn,
        "0.0.0.0",
        22,
        None
    ).map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    mdns.register(service_info).map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;
    st.is_broadcasting = true;

    Ok(())
}

