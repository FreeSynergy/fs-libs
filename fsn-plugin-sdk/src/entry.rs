// run_plugin — stdin/stdout entry point for plugin executables.

use std::io::{self, Read as _, Write as _};

use crate::{CommandRouter, PluginContext, PluginResponse};

/// Entry point for a plugin executable.
///
/// Reads a [`PluginContext`] as JSON from stdin, validates the protocol version,
/// dispatches to the matching command via `router`, and writes a [`PluginResponse`]
/// as JSON to stdout.
///
/// Exits with a non-zero status code if:
/// - stdin cannot be read
/// - the JSON is invalid
/// - the protocol version is not `1`
/// - stdout cannot be written
///
/// Call this as the last statement in your plugin's `main`:
///
/// ```rust,ignore
/// fn main() {
///     let mut router = CommandRouter::new();
///     router.register(DeployCommand);
///     fsn_plugin_sdk::run_plugin(&router);
/// }
/// ```
pub fn run_plugin(router: &CommandRouter) {
    // Read all of stdin
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("fsn-plugin-sdk: failed to read stdin: {e}");
        std::process::exit(1);
    }

    // Deserialize PluginContext
    let ctx: PluginContext = match serde_json::from_str(&input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fsn-plugin-sdk: invalid PluginContext JSON: {e}");
            std::process::exit(1);
        }
    };

    // Validate protocol version
    if ctx.protocol != 1 {
        eprintln!(
            "fsn-plugin-sdk: unsupported protocol version {} (expected 1)",
            ctx.protocol
        );
        std::process::exit(1);
    }

    // Dispatch to the matching command
    let response: PluginResponse = router.dispatch(&ctx);

    // Serialize and write to stdout
    let output = match serde_json::to_string(&response) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fsn-plugin-sdk: failed to serialize PluginResponse: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = io::stdout().write_all(output.as_bytes()) {
        eprintln!("fsn-plugin-sdk: failed to write stdout: {e}");
        std::process::exit(1);
    }
}
