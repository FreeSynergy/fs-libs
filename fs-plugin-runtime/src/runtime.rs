// runtime.rs — WASM engine with WASI sandboxing.
//
// Uses wasmtime + WASI to provide a sandboxed environment for plugins.
// The sandbox restricts filesystem access and disables network access.
//
// WASI capabilities granted per plugin:
//   - stdin/stdout/stderr: always granted (for JSON-RPC fallback)
//   - Filesystem: only paths declared in PluginSandbox::allowed_paths
//   - Network: never granted (plugins use host-shell for external calls)
//   - Environment variables: only explicitly listed keys
//   - Clocks + randomness: granted (harmless)

use std::path::{Path, PathBuf};

use fs_error::FsError;
use fs_plugin_sdk::PluginManifest;
use wasmtime::{Engine, Instance, Linker, Module, Store, Val};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;

use crate::handle::{PluginHandle, PluginInstanceState};

// ── PluginSandbox ─────────────────────────────────────────────────────────────

/// Capability set for a WASM plugin instance.
///
/// Controls which filesystem paths and environment variables the plugin
/// may access. By default everything is denied.
#[derive(Debug, Clone, Default)]
pub struct PluginSandbox {
    /// Filesystem paths the plugin may read from (preopened read-only).
    pub read_paths: Vec<PathBuf>,

    /// Filesystem paths the plugin may write to (preopened read-write).
    pub write_paths: Vec<PathBuf>,

    /// Environment variables exposed to the plugin.
    ///
    /// Only explicitly listed keys are passed — no ambient environment.
    pub env_vars: Vec<(String, String)>,
}

impl PluginSandbox {
    /// Create a minimal sandbox: no filesystem access, no env vars.
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Allow reading from `path`.
    pub fn allow_read(mut self, path: impl Into<PathBuf>) -> Self {
        self.read_paths.push(path.into());
        self
    }

    /// Allow reading and writing at `path`.
    pub fn allow_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.write_paths.push(path.into());
        self
    }

    /// Expose an environment variable to the plugin.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Build a WASI context from this sandbox configuration.
    fn build_wasi_ctx(&self) -> WasiP1Ctx {
        let mut builder = WasiCtxBuilder::new();

        // Always allow stdout/stderr so plugins can emit diagnostics
        builder.inherit_stdout();
        builder.inherit_stderr();

        // Environment variables
        for (k, v) in &self.env_vars {
            builder.env(k, v);
        }

        // NOTE: Preopened directory access via WASI is added here when
        // cap-std is available. For now, plugins use host-shell/host-fs
        // interfaces (defined in wit/) for filesystem access instead.
        // Preopening will be wired up once the Component Model runtime
        // (wasmtime-wasi with WasiView) is fully adopted.

        builder.build_p1()
    }
}

// ── PluginRuntime ─────────────────────────────────────────────────────────────

/// WASM plugin runtime with WASI sandboxing.
///
/// Loads and executes WASM plugins in a capability-limited environment.
/// Each plugin instance gets its own `WasiP1Ctx` configured by `PluginSandbox`.
///
/// # Example
///
/// ```rust,ignore
/// use fs_plugin_runtime::{PluginRuntime, PluginSandbox};
/// use fs_plugin_sdk::PluginContext;
/// use std::path::Path;
///
/// let runtime = PluginRuntime::new()?;
/// let sandbox = PluginSandbox::minimal()
///     .allow_write("/etc/containers/systemd");
/// let mut handle = runtime.load_file(Path::new("zentinel.wasm"), sandbox)?;
/// let ctx = PluginContext { command: "deploy".into(), ..Default::default() };
/// let response = handle.execute(&ctx)?;
/// ```
pub struct PluginRuntime {
    engine: Engine,
}

impl PluginRuntime {
    /// Create a new runtime with WASI support enabled.
    pub fn new() -> Result<Self, FsError> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Load a plugin from WASM bytes with the given sandbox.
    pub fn load(&self, wasm_bytes: &[u8], sandbox: PluginSandbox) -> Result<PluginHandle, FsError> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| FsError::Plugin(format!("failed to compile WASM module: {e}")))?;
        self.instantiate(module, sandbox)
    }

    /// Load a plugin from a WASM file with the given sandbox.
    pub fn load_file(&self, path: &Path, sandbox: PluginSandbox) -> Result<PluginHandle, FsError> {
        let module = Module::from_file(&self.engine, path).map_err(|e| {
            FsError::Plugin(format!("failed to load WASM file {}: {e}", path.display()))
        })?;
        self.instantiate(module, sandbox)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn instantiate(&self, module: Module, sandbox: PluginSandbox) -> Result<PluginHandle, FsError> {
        let wasi_ctx = sandbox.build_wasi_ctx();
        let mut store: Store<WasiP1Ctx> = Store::new(&self.engine, wasi_ctx);

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&self.engine);

        // Add WASI preview1 host functions (compatible with wasm32-wasi target)
        wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |cx: &mut WasiP1Ctx| cx)
            .map_err(|e| FsError::Plugin(format!("failed to add WASI to linker: {e}")))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| FsError::Plugin(format!("failed to instantiate WASM module: {e}")))?;

        let manifest = read_manifest(&mut store, &instance)?;

        tracing::debug!(
            id      = %manifest.id,
            version = %manifest.version,
            "loaded WASM plugin (WASI sandbox)"
        );

        Ok(PluginHandle::new(
            manifest,
            PluginInstanceState { store, instance },
        ))
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new().expect("failed to create PluginRuntime")
    }
}

// ── Manifest bootstrap ────────────────────────────────────────────────────────

/// Call `plugin_manifest_json()` on the freshly instantiated module.
fn read_manifest(
    store: &mut Store<WasiP1Ctx>,
    instance: &Instance,
) -> Result<PluginManifest, FsError> {
    let manifest_fn = instance
        .get_func(&mut *store, "plugin_manifest_json")
        .ok_or_else(|| FsError::Plugin("WASM export `plugin_manifest_json` not found".into()))?;

    let mut results = vec![Val::I32(0)];
    manifest_fn
        .call(&mut *store, &[], &mut results)
        .map_err(|e| FsError::Plugin(format!("plugin_manifest_json call failed: {e}")))?;

    let ptr = results[0]
        .i32()
        .ok_or_else(|| FsError::Plugin("plugin_manifest_json returned non-i32".into()))?
        as usize;

    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or_else(|| FsError::Plugin("WASM export `memory` not found".into()))?;

    let mem_data = memory.data(&*store);

    let end = mem_data[ptr..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| FsError::Plugin("manifest JSON missing null terminator".into()))?;

    let json_bytes = mem_data[ptr..ptr + end].to_vec();
    let json_str = String::from_utf8(json_bytes)
        .map_err(|e| FsError::Plugin(format!("manifest JSON is not valid UTF-8: {e}")))?;

    let manifest: PluginManifest = serde_json::from_str(&json_str)
        .map_err(|e| FsError::Plugin(format!("failed to parse plugin manifest: {e}")))?;

    // Free the manifest buffer
    if let Some(free_fn) = instance.get_func(&mut *store, "plugin_free") {
        let _ = free_fn.call(
            &mut *store,
            &[Val::I32(ptr as i32), Val::I32(json_str.len() as i32)],
            &mut [],
        );
    }

    Ok(manifest)
}
