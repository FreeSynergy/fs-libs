// ProcessPluginRunner — spawns a native plugin executable and applies its response.
//
// Flow:
//   1. Serialize PluginContext to JSON → write to plugin's stdin
//   2. Wait for the plugin to exit
//   3. Deserialize PluginResponse from stdout
//   4. Validate protocol version + error field
//   5. (Separately) apply(): write OutputFiles, run ShellCommands

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use fs_error::FsError;
use fs_plugin_sdk::{PluginContext, PluginResponse};

/// Process-based plugin runner (fallback for non-WASM plugins).
///
/// Used for plugins that are native executables rather than WASM modules.
/// The plugin executable is expected at `{store_module_dir}/plugin`.
///
/// For WASM plugins, use [`PluginRuntime`][crate::PluginRuntime] instead.
pub struct ProcessPluginRunner {
    /// Root of the Store directory for this module.
    ///
    /// E.g. `/home/user/.local/share/fsn/store/Node/proxy/zentinel`.
    pub store_module_dir: PathBuf,
}

impl ProcessPluginRunner {
    /// Create a new runner pointing at the given Store module directory.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { store_module_dir: dir.into() }
    }

    /// Invoke the plugin with the given context and return its response.
    ///
    /// Returns `Ok(PluginResponse)` on success. Returns `Err` if:
    /// - the plugin executable is not found
    /// - the process fails to spawn or exits non-zero
    /// - stdout is not valid JSON
    /// - `PluginResponse::protocol != 1`
    /// - `PluginResponse::error` is non-empty
    pub fn run(&self, ctx: &PluginContext) -> Result<PluginResponse, FsError> {
        let executable = self.store_module_dir.join("plugin");

        if !executable.exists() {
            return Err(FsError::Plugin(format!(
                "plugin executable not found: {}",
                executable.display()
            )));
        }

        let ctx_json = serde_json::to_string(ctx).map_err(|e| {
            FsError::Plugin(format!("failed to serialize PluginContext: {e}"))
        })?;

        tracing::debug!(
            executable = %executable.display(),
            command = %ctx.command,
            "spawning plugin"
        );

        let mut child = Command::new(&executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // plugin stderr visible in parent's stderr
            .spawn()
            .map_err(|e| {
                FsError::Plugin(format!(
                    "failed to spawn plugin {}: {e}",
                    executable.display()
                ))
            })?;

        // Write context JSON to stdin; drop handle to close the pipe (EOF for plugin)
        {
            let stdin = child.stdin.take().expect("stdin was piped");
            let mut stdin = stdin;
            stdin.write_all(ctx_json.as_bytes()).map_err(|e| {
                FsError::Plugin(format!("failed to write to plugin stdin: {e}"))
            })?;
        }

        let output = child.wait_with_output().map_err(|e| {
            FsError::Plugin(format!("plugin process error: {e}"))
        })?;

        if !output.status.success() {
            return Err(FsError::Plugin(format!(
                "plugin exited with status {} (command: {})",
                output.status, ctx.command
            )));
        }

        let response: PluginResponse =
            serde_json::from_slice(&output.stdout).map_err(|e| {
                FsError::Plugin(format!("invalid JSON from plugin stdout: {e}"))
            })?;

        if response.protocol != 1 {
            return Err(FsError::Plugin(format!(
                "unsupported plugin protocol version {} (expected 1)",
                response.protocol
            )));
        }

        if response.has_error() {
            return Err(FsError::Plugin(format!("plugin error: {}", response.error)));
        }

        Ok(response)
    }

    /// Apply a [`PluginResponse`]: write declared files then run declared shell commands.
    ///
    /// Call this after [`run`][Self::run] once you've decided to commit the changes.
    pub fn apply(&self, response: &PluginResponse) -> Result<(), FsError> {
        self.write_files(response)?;
        self.run_commands(response)?;
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn write_files(&self, response: &PluginResponse) -> Result<(), FsError> {
        for file in &response.files {
            let dest = Path::new(&file.dest);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    FsError::Plugin(format!(
                        "failed to create directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }

            fs::write(dest, &file.content).map_err(|e| {
                FsError::Plugin(format!("failed to write {}: {e}", dest.display()))
            })?;

            fs::set_permissions(dest, fs::Permissions::from_mode(file.mode)).map_err(
                |e| {
                    FsError::Plugin(format!(
                        "failed to set permissions on {}: {e}",
                        dest.display()
                    ))
                },
            )?;

            tracing::debug!(dest = %dest.display(), mode = file.mode, "wrote output file");
        }
        Ok(())
    }

    fn run_commands(&self, response: &PluginResponse) -> Result<(), FsError> {
        for shell_cmd in &response.commands {
            let mut builder = Command::new("sh");
            builder.arg("-c").arg(&shell_cmd.cmd);

            if let Some(cwd) = &shell_cmd.cwd {
                builder.current_dir(cwd);
            }

            for (k, v) in &shell_cmd.env {
                builder.env(k, v);
            }

            tracing::debug!(cmd = %shell_cmd.cmd, "running shell command");

            let status = builder.status().map_err(|e| {
                FsError::Plugin(format!(
                    "failed to run command `{}`: {e}",
                    shell_cmd.cmd
                ))
            })?;

            if !status.success() {
                return Err(FsError::Plugin(format!(
                    "command failed (exit {}): {}",
                    status.code().unwrap_or(-1),
                    shell_cmd.cmd
                )));
            }
        }
        Ok(())
    }
}
