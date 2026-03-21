# fs-sync

CRDT sync layer for FreeSynergy. Wraps [Automerge](https://automerge.org/) for
conflict-free distributed state synchronization over any transport.

## Core Types

| Type | Role |
|---|---|
| `SyncDoc<T>` | CRDT document holding one serializable value |
| `SyncPeer` | Sync-state tracker for one remote peer |
| `SyncEngine` | Factory: creates docs and peers for an actor |
| `WsTransport` | WebSocket transport (`ws` feature) |
| `SyncSession<T>` | Ties a `SyncDoc` to a `WsTransport` for live sync |
| `AuditEntry` | Records changes with timestamp + actor |

## Usage

```rust
use fs_sync::{SyncDoc, SyncEngine, SyncPeer};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config { name: String, port: u16 }

let engine = SyncEngine::new("node-a".into());
let mut doc: SyncDoc<Config> = engine.new_doc();
doc.set(&Config { name: "test".into(), port: 8080 }).unwrap();

let mut peer = engine.new_peer("node-b");
let msg = doc.generate_sync_message(&mut peer);
```

## WebSocket Sync (`ws` feature)

```rust
use fs_sync::{SyncSession, WsTransport};

let transport = WsTransport::connect("ws://peer.example.com/sync").await?;
let mut session: SyncSession<Config> = SyncSession::new(doc, transport);
session.run_until_synced().await?;
```

## Features

| Flag | Enables |
|---|---|
| `ws` | `WsTransport` + `SyncSession` via `tokio-tungstenite` |
