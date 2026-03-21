// fsn-sync — CRDT sync layer for FreeSynergy.
//
// Wraps Automerge for conflict-free distributed state sync.
// Values are stored as JSON strings inside an Automerge document.
//
// Design:
//   SyncDoc<T>   — CRDT document holding one serializable value
//   SyncPeer     — sync state tracker for one remote peer
//   SyncEngine   — factory + coordinator for docs and peers
//   AuditEntry   — records changes with timestamp + actor
//   WsTransport  — WebSocket binary transport (feature `ws`)
//   SyncSession  — live sync over a WsTransport (feature `ws`)
//
// Pattern: Facade (SyncDoc hides Automerge complexity),
//          Strategy (SyncPeer can be reset without recreating the doc)

#[cfg(feature = "ws")]
pub mod transport;

#[cfg(feature = "ws")]
pub use transport::{SyncSession, WsTransport};

use automerge::{transaction::Transactable, AutoCommit, ReadDoc, ScalarValue, ROOT};
use fsn_error::FsnError;
use serde::{Deserialize, Serialize};

// ── ActorId ───────────────────────────────────────────────────────────────────

/// Unique identifier for a sync participant.
pub type ActorId = String;

// ── SyncPeer ──────────────────────────────────────────────────────────────────

/// Tracks sync state with one remote peer.
///
/// One `SyncPeer` is needed per (local_doc, remote_peer) pair.
pub struct SyncPeer {
    state: automerge::sync::State,
    peer_id: String,
}

impl SyncPeer {
    /// Create a new peer tracker.
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self { state: automerge::sync::State::new(), peer_id: peer_id.into() }
    }

    /// The peer's identifier.
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Reset sync state — call this after reconnecting to a peer.
    pub fn reset(&mut self) {
        self.state = automerge::sync::State::new();
    }
}

// ── SyncDoc ───────────────────────────────────────────────────────────────────

const VALUE_KEY: &str = "value";

/// A CRDT-backed document holding a value of type `T`.
///
/// The value is stored as a JSON string at key `"value"` inside an Automerge
/// document, giving conflict-free merge semantics.
pub struct SyncDoc<T> {
    doc: AutoCommit,
    actor: ActorId,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de>> SyncDoc<T> {
    /// Create a new empty document.
    pub fn new(actor: ActorId) -> Self {
        let doc = AutoCommit::new();
        Self { doc, actor, _marker: std::marker::PhantomData }
    }

    /// Create a document and initialize it with `value`.
    pub fn with_value(actor: ActorId, value: &T) -> Result<Self, FsnError> {
        let mut s = Self::new(actor);
        s.set(value)?;
        Ok(s)
    }

    /// Deserialize the current value from the CRDT.
    pub fn get(&self) -> Result<T, FsnError> {
        let item = self.doc
            .get(ROOT, VALUE_KEY)
            .map_err(|e| FsnError::internal(e.to_string()))?
            .ok_or_else(|| FsnError::NotFound("sync doc has no value".into()))?;

        let json = match item.0 {
            automerge::Value::Scalar(s) => match s.as_ref() {
                ScalarValue::Str(arc_str) => arc_str.to_string(),
                other => return Err(FsnError::Parse(format!("unexpected scalar: {other:?}"))),
            },
            other => return Err(FsnError::Parse(format!("unexpected node: {other:?}"))),
        };

        serde_json::from_str(&json)
            .map_err(|e| FsnError::Parse(format!("sync deserialize: {e}")))
    }

    /// Update the value — creates a new CRDT change.
    pub fn set(&mut self, value: &T) -> Result<(), FsnError> {
        let json = serde_json::to_string(value)
            .map_err(|e| FsnError::Parse(format!("sync serialize: {e}")))?;
        self.doc
            .put(ROOT, VALUE_KEY, json)
            .map_err(|e| FsnError::internal(e.to_string()))?;
        self.doc.commit();
        Ok(())
    }

    /// Generate a sync message to send to `peer`. Returns `None` if nothing new.
    pub fn generate_sync_message(&mut self, peer: &mut SyncPeer) -> Option<Vec<u8>> {
        use automerge::sync::SyncDoc;
        self.doc.sync().generate_sync_message(&mut peer.state).map(|msg| msg.encode())
    }

    /// Apply a sync message received from `peer`.
    pub fn receive_sync_message(&mut self, peer: &mut SyncPeer, bytes: &[u8]) -> Result<(), FsnError> {
        use automerge::sync::SyncDoc;
        let msg = automerge::sync::Message::decode(bytes)
            .map_err(|e| FsnError::internal(format!("sync decode: {e}")))?;
        self.doc
            .sync()
            .receive_sync_message(&mut peer.state, msg)
            .map_err(|e| FsnError::internal(format!("sync apply: {e}")))
    }

    /// Serialize the full document for backup or initial handshake.
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Load a document from previously saved bytes.
    pub fn load(actor: ActorId, bytes: &[u8]) -> Result<Self, FsnError> {
        let doc = AutoCommit::load(bytes)
            .map_err(|e| FsnError::internal(format!("sync load: {e}")))?;
        Ok(Self { doc, actor, _marker: std::marker::PhantomData })
    }

    /// The actor ID this document belongs to.
    pub fn actor(&self) -> &str {
        &self.actor
    }
}

// ── SyncEngine ────────────────────────────────────────────────────────────────

/// Factory and coordinator for sync documents and peers.
pub struct SyncEngine {
    actor: ActorId,
}

impl SyncEngine {
    /// Create a new engine for the given actor.
    pub fn new(actor: ActorId) -> Self {
        Self { actor }
    }

    /// The actor ID.
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Create a new empty sync document owned by this engine's actor.
    pub fn new_doc<T: Serialize + for<'de> Deserialize<'de>>(&self) -> SyncDoc<T> {
        SyncDoc::new(self.actor.clone())
    }

    /// Create a new peer tracker.
    pub fn new_peer(&self, peer_id: impl Into<String>) -> SyncPeer {
        SyncPeer::new(peer_id)
    }
}

// ── AuditEntry ────────────────────────────────────────────────────────────────

/// A single recorded change — for audit logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// The actor that made the change.
    pub actor: ActorId,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Document identifier.
    pub doc_id: String,
    /// Human-readable description.
    pub description: String,
}

impl AuditEntry {
    /// Create an entry with the current system time.
    pub fn now(actor: ActorId, doc_id: impl Into<String>, description: impl Into<String>) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self { actor, timestamp_ms, doc_id: doc_id.into(), description: description.into() }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Cfg { name: String, port: u16 }

    #[test]
    fn roundtrip() {
        let cfg = Cfg { name: "test".into(), port: 8080 };
        let mut doc: SyncDoc<Cfg> = SyncDoc::with_value("alice".into(), &cfg).unwrap();
        assert_eq!(doc.get().unwrap(), cfg);
        let updated = Cfg { name: "updated".into(), port: 9090 };
        doc.set(&updated).unwrap();
        assert_eq!(doc.get().unwrap(), updated);
    }

    #[test]
    fn save_and_load() {
        let cfg = Cfg { name: "saved".into(), port: 1234 };
        let mut doc: SyncDoc<Cfg> = SyncDoc::with_value("bob".into(), &cfg).unwrap();
        let bytes = doc.save();
        let loaded: SyncDoc<Cfg> = SyncDoc::load("bob".into(), &bytes).unwrap();
        assert_eq!(loaded.get().unwrap(), cfg);
    }

    #[test]
    fn sync_two_peers() {
        let cfg_a = Cfg { name: "from_alice".into(), port: 1 };
        let cfg_b = Cfg { name: "from_bob".into(), port: 2 };

        let mut doc_a: SyncDoc<Cfg> = SyncDoc::with_value("alice".into(), &cfg_a).unwrap();
        let mut doc_b: SyncDoc<Cfg> = SyncDoc::new("bob".into());
        doc_b.set(&cfg_b).unwrap();

        let mut peer_b_in_a = SyncPeer::new("bob");
        let mut peer_a_in_b = SyncPeer::new("alice");

        // Exchange messages until both sides have nothing new
        for _ in 0..5 {
            if let Some(msg) = doc_a.generate_sync_message(&mut peer_b_in_a) {
                doc_b.receive_sync_message(&mut peer_a_in_b, &msg).unwrap();
            }
            if let Some(msg) = doc_b.generate_sync_message(&mut peer_a_in_b) {
                doc_a.receive_sync_message(&mut peer_b_in_a, &msg).unwrap();
            }
        }

        let val_a = doc_a.get().unwrap();
        let val_b = doc_b.get().unwrap();
        assert_eq!(val_a, val_b, "docs must converge");
    }
}
