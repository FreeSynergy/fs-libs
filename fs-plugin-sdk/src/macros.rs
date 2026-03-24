// plugin_main! — entry point macro for WASM plugins.

/// Entry point macro for WASM plugins.
///
/// Generates the WASM exports expected by the `fs-plugin-runtime` host:
/// - `plugin_manifest_json() -> *mut u8` — returns manifest as null-terminated JSON
/// - `plugin_execute(cmd_ptr, cmd_len, ctx_ptr, ctx_len) -> *mut u8` — executes a command
/// - `plugin_alloc(len) -> *mut u8` — lets the host allocate WASM memory for inputs
/// - `plugin_free(ptr, len)` — releases a buffer returned by this plugin
///
/// The plugin type must implement both [`PluginImpl`][crate::PluginImpl] and [`Default`].
///
/// # Example
///
/// ```rust,ignore
/// use fs_plugin_sdk::{plugin_main, PluginImpl, PluginManifest, PluginContext, PluginResponse};
///
/// #[derive(Default)]
/// struct MyPlugin;
///
/// impl PluginImpl for MyPlugin {
///     fn manifest(&self) -> PluginManifest { /* … */ }
///     fn execute(&self, command: &str, ctx: &PluginContext) -> Result<PluginResponse, String> {
///         Ok(PluginResponse::default())
///     }
/// }
///
/// plugin_main!(MyPlugin);
/// ```
#[macro_export]
macro_rules! plugin_main {
    ($plugin_type:ty) => {
        static PLUGIN: std::sync::OnceLock<$plugin_type> = std::sync::OnceLock::new();

        fn get_plugin() -> &'static $plugin_type {
            PLUGIN.get_or_init(|| <$plugin_type>::default())
        }

        /// WASM export: return the plugin manifest as a null-terminated JSON string.
        ///
        /// The host must call `plugin_free` with the returned pointer and the
        /// length of the JSON string (excluding the null terminator) when done.
        #[no_mangle]
        pub extern "C" fn plugin_manifest_json() -> *mut u8 {
            let manifest = $crate::PluginImpl::manifest(get_plugin());
            let json = serde_json::to_string(&manifest).unwrap_or_default();
            let mut buf = json.into_bytes();
            buf.push(0); // null terminator
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        /// WASM export: execute a command with a JSON-serialized `PluginContext`.
        ///
        /// Returns a null-terminated JSON string containing either a `PluginResponse`
        /// (on success) or a `PluginResponse::err(…)` (on failure). The host must
        /// call `plugin_free` with the returned pointer and JSON length when done.
        #[no_mangle]
        pub extern "C" fn plugin_execute(
            command_ptr: *const u8,
            command_len: usize,
            context_ptr: *const u8,
            context_len: usize,
        ) -> *mut u8 {
            let command = unsafe {
                let slice = std::slice::from_raw_parts(command_ptr, command_len);
                String::from_utf8_lossy(slice).into_owned()
            };
            let context_json = unsafe {
                let slice = std::slice::from_raw_parts(context_ptr, context_len);
                String::from_utf8_lossy(slice).into_owned()
            };

            let context: $crate::PluginContext = match serde_json::from_str(&context_json) {
                Ok(c) => c,
                Err(e) => {
                    let response =
                        $crate::PluginResponse::err(format!("invalid context JSON: {e}"));
                    let json = serde_json::to_string(&response).unwrap_or_default();
                    let mut buf = json.into_bytes();
                    buf.push(0);
                    let ptr = buf.as_mut_ptr();
                    std::mem::forget(buf);
                    return ptr;
                }
            };

            let result = $crate::PluginImpl::execute(get_plugin(), &command, &context);
            let response = match result {
                Ok(r) => r,
                Err(msg) => $crate::PluginResponse::err(msg),
            };

            let json = serde_json::to_string(&response).unwrap_or_default();
            let mut buf = json.into_bytes();
            buf.push(0);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        /// WASM export: allocate a buffer for the host to write input data into.
        ///
        /// The host calls this before writing command and context strings into
        /// the plugin's linear memory. The plugin owns the allocation; the host
        /// must not free memory returned by `plugin_alloc` directly.
        #[no_mangle]
        pub extern "C" fn plugin_alloc(len: usize) -> *mut u8 {
            let mut buf = Vec::with_capacity(len);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        /// WASM export: free a buffer that was allocated by this plugin.
        ///
        /// Called by the host after it has finished reading a string returned by
        /// `plugin_manifest_json` or `plugin_execute`.
        #[no_mangle]
        pub extern "C" fn plugin_free(ptr: *mut u8, len: usize) {
            if !ptr.is_null() && len > 0 {
                unsafe {
                    let _ = Vec::from_raw_parts(ptr, len, len);
                }
            }
        }
    };
}
