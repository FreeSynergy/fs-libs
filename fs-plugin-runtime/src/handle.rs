// PluginHandle — a loaded, initialized WASM plugin instance.

use fs_error::FsError;
use fs_plugin_sdk::{PluginContext, PluginManifest, PluginResponse};
use wasmtime::{Instance, Store, Val};
use wasmtime_wasi::preview1::WasiP1Ctx;

// ── PluginInstanceState ───────────────────────────────────────────────────────

/// Internal wasmtime state for a loaded WASM plugin.
pub(crate) struct PluginInstanceState {
    pub(crate) store: Store<WasiP1Ctx>,
    pub(crate) instance: Instance,
}

// ── PluginHandle ──────────────────────────────────────────────────────────────

/// A loaded, initialized WASM plugin ready to execute commands.
///
/// Obtain a `PluginHandle` via [`PluginRuntime::load`][crate::PluginRuntime::load]
/// or [`PluginRuntime::load_file`][crate::PluginRuntime::load_file].
pub struct PluginHandle {
    manifest: PluginManifest,
    state: PluginInstanceState,
}

impl PluginHandle {
    /// Create a new handle from a manifest and instantiated WASM state.
    pub(crate) fn new(manifest: PluginManifest, state: PluginInstanceState) -> Self {
        Self { manifest, state }
    }

    /// Plugin identifier from the manifest (e.g. `"zentinel"`).
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Plugin semantic version string (e.g. `"0.1.0"`).
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Full plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Invoke a command on the plugin.
    ///
    /// Serializes `input` as JSON, writes it into WASM memory, calls the
    /// plugin's `plugin_execute` export, and deserializes the response.
    ///
    /// Returns `Err` if the WASM call fails or the plugin returns an error response.
    pub fn execute(&mut self, input: &PluginContext) -> Result<PluginResponse, FsError> {
        let command = input.command.clone();
        let context_json = serde_json::to_string(input)
            .map_err(|e| FsError::Plugin(format!("failed to serialize PluginContext: {e}")))?;

        let store = &mut self.state.store;
        let instance = self.state.instance;

        // Allocate WASM memory for command string
        let cmd_bytes = command.as_bytes().to_vec();
        let cmd_ptr = wasm_alloc(store, &instance, cmd_bytes.len())?;
        wasm_write(store, &instance, cmd_ptr, &cmd_bytes)?;

        // Allocate WASM memory for context JSON
        let ctx_bytes = context_json.as_bytes().to_vec();
        let ctx_ptr = wasm_alloc(store, &instance, ctx_bytes.len())?;
        wasm_write(store, &instance, ctx_ptr, &ctx_bytes)?;

        // Call plugin_execute
        let execute_fn = instance
            .get_func(&mut *store, "plugin_execute")
            .ok_or_else(|| FsError::Plugin("WASM export `plugin_execute` not found".into()))?;

        let mut results = vec![Val::I32(0)];
        execute_fn
            .call(
                &mut *store,
                &[
                    Val::I32(cmd_ptr as i32),
                    Val::I32(cmd_bytes.len() as i32),
                    Val::I32(ctx_ptr as i32),
                    Val::I32(ctx_bytes.len() as i32),
                ],
                &mut results,
            )
            .map_err(|e| FsError::Plugin(format!("plugin_execute call failed: {e}")))?;

        let result_ptr = results[0]
            .i32()
            .ok_or_else(|| FsError::Plugin("plugin_execute returned non-i32".into()))?
            as usize;

        // Read null-terminated JSON from WASM memory
        let json_str = wasm_read_cstr(&mut *store, &instance, result_ptr)?;
        let json_len = json_str.len();

        // Free the result buffer
        wasm_free(&mut *store, &instance, result_ptr, json_len);

        let response: PluginResponse = serde_json::from_str(&json_str)
            .map_err(|e| FsError::Plugin(format!("invalid JSON from plugin_execute: {e}")))?;

        if response.has_error() {
            return Err(FsError::Plugin(format!(
                "plugin '{}' error: {}",
                self.manifest.id, response.error
            )));
        }

        Ok(response)
    }
}

// ── WASM memory helpers ───────────────────────────────────────────────────────

/// Allocate `len` bytes in the WASM module's linear memory via `plugin_alloc`.
fn wasm_alloc(
    store: &mut Store<WasiP1Ctx>,
    instance: &Instance,
    len: usize,
) -> Result<usize, FsError> {
    let alloc_fn = instance
        .get_func(&mut *store, "plugin_alloc")
        .ok_or_else(|| FsError::Plugin("WASM export `plugin_alloc` not found".into()))?;

    let mut results = vec![Val::I32(0)];
    alloc_fn
        .call(&mut *store, &[Val::I32(len as i32)], &mut results)
        .map_err(|e| FsError::Plugin(format!("plugin_alloc call failed: {e}")))?;

    let ptr = results[0]
        .i32()
        .ok_or_else(|| FsError::Plugin("plugin_alloc returned non-i32".into()))?
        as usize;

    Ok(ptr)
}

/// Write `data` into the WASM module's linear memory at `offset`.
fn wasm_write(
    store: &mut Store<WasiP1Ctx>,
    instance: &Instance,
    offset: usize,
    data: &[u8],
) -> Result<(), FsError> {
    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or_else(|| FsError::Plugin("WASM export `memory` not found".into()))?;

    memory
        .write(&mut *store, offset, data)
        .map_err(|e| FsError::Plugin(format!("WASM memory write failed: {e}")))?;

    Ok(())
}

/// Read a null-terminated UTF-8 string from the WASM module's linear memory.
fn wasm_read_cstr(
    store: &mut Store<WasiP1Ctx>,
    instance: &Instance,
    offset: usize,
) -> Result<String, FsError> {
    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or_else(|| FsError::Plugin("WASM export `memory` not found".into()))?;

    let mem_data = memory.data(&*store);

    // Find null terminator
    let end = mem_data[offset..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| FsError::Plugin("WASM string missing null terminator".into()))?;

    let bytes = mem_data[offset..offset + end].to_vec();
    String::from_utf8(bytes)
        .map_err(|e| FsError::Plugin(format!("WASM string is not valid UTF-8: {e}")))
}

/// Free a buffer in the WASM module's linear memory via `plugin_free`.
///
/// Errors are logged but not propagated — freeing is best-effort.
fn wasm_free(store: &mut Store<WasiP1Ctx>, instance: &Instance, ptr: usize, len: usize) {
    if let Some(free_fn) = instance.get_func(&mut *store, "plugin_free") {
        let _ = free_fn.call(
            store,
            &[Val::I32(ptr as i32), Val::I32(len as i32)],
            &mut [],
        );
    }
}
