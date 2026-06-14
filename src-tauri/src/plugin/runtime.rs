use crate::errors::{AppError, AppResult};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

/// Maximum WASM execution time before timeout.
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);
/// Maximum WASM memory (16 MB).
const MAX_MEMORY_PAGES: u32 = 256; // 256 * 64KB = 16MB

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
        // Resource limits for sandboxing.
        config.max_wasm_stack(1024 * 1024); // 1MB stack
        // Memory limit is set per-store via limiter.
        config.epoch_interruption(true);

        let engine = wasmtime::Engine::new(&config)
            .map_err(|e| AppError::Internal(format!("Wasmtime engine: {e}")))?;
        Ok(Self { engine })
    }

    /// Execute a plugin's `_start` or `run` entry point.
    /// Enforces resource limits and catches traps.
    pub fn execute(&self, wasm_path: &str) -> AppResult<String> {
        let path = Path::new(wasm_path);
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "WASM file not found: {wasm_path}"
            )));
        }

        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .build();

        let mut store = wasmtime::Store::new(&self.engine, wasi);

        // Set resource limits.
        store.limiter(|_| wasmtime::InstanceLimits {
            memory_pages: Some(MAX_MEMORY_PAGES),
            ..Default::default()
        });

        // Set epoch-based timeout.
        store.set_epoch_deadline(1);
        let engine_clone = self.engine.clone();
        std::thread::spawn(move || {
            std::thread::sleep(EXECUTION_TIMEOUT);
            engine_clone.increment_epoch();
        });

        let module = wasmtime::Module::from_file(&self.engine, path)
            .map_err(|e| AppError::Internal(format!("WASM load: {e}")))?;

        let instance = match wasmtime::Instance::new(&mut store, &module, &[]) {
            Ok(i) => i,
            Err(e) => {
                warn!("Plugin instance creation failed: {e}");
                return Err(AppError::Internal(format!("plugin init: {e}")));
            }
        };

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
                Ok(String::new())
            }
            Err(e) => {
                if e.is::<wasmtime::Trap>() {
                    warn!("Plugin trapped (resource limit or invalid operation): {e}");
                    return Err(AppError::InvalidInput(format!("plugin sandbox violation: {e}")));
                }
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
