# fsn-plugin-runtime

WASM host runtime for loading and executing FreeSynergy plugins.
Uses [wasmtime](https://wasmtime.dev/) with WASI sandboxing (`wasm` feature) or
spawns native process plugins via `ProcessPluginRunner`.

## Features

| Feature | Default | Description |
|---|---|---|
| `wasm`  | no | Enable wasmtime WASM execution + WASI sandbox |

## WASM plugin (`wasm` feature)

```rust
use fsn_plugin_runtime::{PluginRuntime, PluginSandbox};
use fsn_plugin_sdk::PluginContext;
use std::path::Path;

let runtime = PluginRuntime::new()?;

// Fine-grained sandbox capabilities
let sandbox = PluginSandbox::minimal()
    .allow_write("/etc/containers/systemd")
    .with_env("DATA_ROOT", "/srv/data");

let mut handle = runtime.load_file(Path::new("zentinel.wasm"), sandbox)?;

let ctx = PluginContext {
    protocol: 1,
    command: "deploy".into(),
    instance: /* … */,
    peers: vec![],
    env: Default::default(),
};

let response = handle.execute(&ctx)?;

for file in &response.files {
    println!("write → {}", file.dest);
}
```

## Native process plugin (always available)

```rust
use fsn_plugin_runtime::ProcessPluginRunner;
use fsn_plugin_sdk::PluginContext;

let runner = ProcessPluginRunner::new("/store/Node/proxy/zentinel");
let response = runner.run(&ctx)?;
runner.apply(&response)?;   // writes files + runs shell commands
```

## Sandbox model

The WASI sandbox grants:
- **stdout/stderr**: always (for diagnostics)
- **Filesystem**: only paths in `PluginSandbox::read_paths` / `write_paths`
- **Network**: never (plugins use `ShellCommand` for outbound calls)
- **Env vars**: only explicitly listed keys
