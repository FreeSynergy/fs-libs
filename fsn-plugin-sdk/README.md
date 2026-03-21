# fsn-plugin-sdk

SDK for authoring FreeSynergy plugins. Compile your plugin to `wasm32-wasi`
and the host runtime loads it in a sandboxed environment.

## Quick start (WASM plugin)

```rust
use fsn_plugin_sdk::{plugin_main, PluginContext, PluginImpl, PluginManifest, PluginResponse};

#[derive(Default)]
struct ZentinelPlugin;

impl PluginImpl for ZentinelPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id:          "zentinel".into(),
            version:     "0.1.0".into(),
            commands:    vec!["deploy".into(), "clean".into()],
            description: "Zentinel reverse proxy plugin".into(),
        }
    }

    fn execute(&self, command: &str, ctx: &PluginContext) -> Result<PluginResponse, String> {
        match command {
            "deploy" => Ok(PluginResponse::default()),
            "clean"  => Ok(PluginResponse::default()),
            other    => Err(format!("unknown command: {other}")),
        }
    }
}

plugin_main!(ZentinelPlugin);
```

Compile with:

```sh
cargo build --target wasm32-wasi --release
```

## Native process plugin

For non-WASM plugins (native executables), use `CommandRouter` + `run_plugin`:

```rust
use fsn_plugin_sdk::{CommandRouter, PluginCommand, PluginContext, PluginResponse, run_plugin};

struct DeployCommand;
impl PluginCommand for DeployCommand {
    fn name(&self) -> &str { "deploy" }
    fn execute(&self, _ctx: &PluginContext) -> PluginResponse { PluginResponse::default() }
}

fn main() {
    let mut router = CommandRouter::new();
    router.register(DeployCommand);
    run_plugin(&router);
}
```

## Protocol

The host writes a JSON `PluginContext` to the plugin's stdin, and the plugin
writes a JSON `PluginResponse` to stdout. Protocol version must be `1`.

### `PluginResponse` fields

| Field | Type | Description |
|---|---|---|
| `logs` | `Vec<LogLine>` | Messages to surface in the host UI |
| `files` | `Vec<OutputFile>` | Files to write (host writes them) |
| `commands` | `Vec<ShellCommand>` | Shell commands for the host to run |
| `error` | `String` | Non-empty → host aborts with this message |
