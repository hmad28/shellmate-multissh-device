use crate::errors::{AppError, AppResult};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub struct PluginRuntime {
    engine: wasmtime::Engine,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl PluginRuntime {
    pub fn new() -> AppResult<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(false);
        config.max_wasm_stack(1024 * 1024);
        config.epoch_interruption(true);
        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| AppError::Internal(format!("Wasmtime engine: {e}")))?;
        Ok(Self { engine })
    }

    pub fn execute(&self, wasm_path: &str) -> AppResult<String> {
        let path = Path::new(wasm_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "WASM file not found: {wasm_path}"
            )));
        }

        let wasi = wasmtime_wasi::WasiCtxBuilder::new().build();
        let mut store = wasmtime::Store::new(&self.engine, wasi);
        store.set_epoch_deadline(1);

        let engine_clone = self.engine.clone();
        std::thread::spawn(move || {
            std::thread::sleep(EXECUTION_TIMEOUT);
            engine_clone.increment_epoch();
        });

        let module = wasmtime::Module::from_file(&self.engine, path)
            .map_err(|e| AppError::Internal(format!("WASM load: {e}")))?;
        if module.imports().next().is_some() {
            return Err(AppError::InvalidInput(
                "plugins with WASI/imported host functions are not supported yet".into(),
            ));
        }

        let instance = match wasmtime::Instance::new(&mut store, &module, &[]) {
            Ok(i) => i,
            Err(e) => {
                warn!("Plugin instance creation failed: {e}");
                return Err(AppError::Internal(format!("plugin init: {e}")));
            }
        };

        let result = if let Some(start) = instance.get_func(&mut store, "_start") {
            start.call(&mut store, &[], &mut [])
        } else if let Some(run) = instance.get_func(&mut store, "run") {
            run.call(&mut store, &[], &mut [])
        } else {
            return Err(AppError::InvalidInput(
                "plugin has no '_start' or 'run' function".into(),
            ));
        };

        match result {
            Ok(_) => {
                info!("Plugin executed successfully: {wasm_path}");
                Ok(String::new())
            }
            Err(e) => {
                if e.is::<wasmtime::Trap>() {
                    warn!("Plugin trapped: {e}");
                    return Err(AppError::InvalidInput(format!(
                        "plugin sandbox violation: {e}"
                    )));
                }
                warn!("Plugin execution error: {e}");
                Err(AppError::Internal(format!("plugin runtime: {e}")))
            }
        }
    }

    pub fn validate(&self, wasm_path: &str) -> AppResult<()> {
        let path = Path::new(wasm_path);
        let module = wasmtime::Module::from_file(&self.engine, path)
            .map_err(|e| AppError::InvalidInput(format!("invalid WASM: {e}")))?;
        if module.imports().next().is_some() {
            return Err(AppError::InvalidInput(
                "plugins with WASI/imported host functions are not supported yet".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub struct PluginRuntime;

#[cfg(any(target_os = "android", target_os = "ios"))]
impl PluginRuntime {
    pub fn new() -> AppResult<Self> {
        Ok(Self)
    }

    pub fn execute(&self, _wasm_path: &str) -> AppResult<String> {
        Err(AppError::Internal(
            "Plugin execution is not supported on mobile".into(),
        ))
    }

    pub fn validate(&self, _wasm_path: &str) -> AppResult<()> {
        Err(AppError::Internal(
            "Plugin validation is not supported on mobile".into(),
        ))
    }
}
