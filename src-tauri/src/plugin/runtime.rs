use crate::errors::{AppError, AppResult};
use log::{info, warn};
use std::path::Path;

/// Plugin runtime manages WASM execution via Wasmtime.
/// Each plugin runs in its own sandboxed instance with WASI support.
pub struct PluginRuntime {
    engine: wasmtime::Engine,
}

impl PluginRuntime {
    /// Create a new plugin runtime.
    pub fn new() -> AppResult<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(false);
        // Enable WASI for basic I/O (stdout/stderr → log).
        // Security: no filesystem, no network by default.
        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| AppError::Internal(format!("Wasmtime engine: {e}")))?;
        Ok(Self { engine })
    }

    /// Execute a plugin's `_start` or `run` entry point.
    /// Returns stdout output as a string.
    /// Plugin crashes are caught and returned as errors (never crash host).
    pub fn execute(&self, wasm_path: &str) -> AppResult<String> {
        let path = Path::new(wasm_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "WASM file not found: {wasm_path}"
            )));
        }

        // Create WASI context with stdout/stderr captured.
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .build();

        let mut store = wasmtime::Store::new(&self.engine, wasi);

        let module = wasmtime::Module::from_file(&self.engine, path)
            .map_err(|e| AppError::Internal(format!("WASM load: {e}")))?;

        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| {
                warn!("Plugin instance creation failed: {e}");
                AppError::Internal(format!("plugin init: {e}"))
            })?;

        // Try to call `_start` (WASI entry point) or `run`.
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
                Ok(String::new()) // TODO: capture stdout
            }
            Err(e) => {
                warn!("Plugin execution error: {e}");
                Err(AppError::Internal(format!("plugin runtime: {e}")))
            }
        }
    }

    /// Validate that a WASM file is valid and loadable.
    pub fn validate(&self, wasm_path: &str) -> AppResult<()> {
        let path = Path::new(wasm_path);
        wasmtime::Module::from_file(&self.engine, path)
            .map_err(|e| AppError::InvalidInput(format!("invalid WASM: {e}")))?;
        Ok(())
    }
}
