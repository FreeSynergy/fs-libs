// transport.rs — WebSocket transport for CRDT sync messages.
//
// Enabled by the `ws` feature flag.
//
// Design:
//   WsTransport  — wraps a tokio-tungstenite WebSocket connection
//   SyncSession<T> — ties a SyncDoc + SyncPeer to a WsTransport for live sync
//
// Protocol:
//   - Messages are raw binary frames carrying Automerge sync payloads.
//   - Either side may send a sync message at any time.
//   - `run_until_synced` exchanges messages until neither side has anything new
//     to send (both produce `None` from `generate_sync_message`).
//   - After sync converges, the session sends a Close frame and returns.

use std::sync::Arc;

use fsn_error::FsnError;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, warn};

use crate::{SyncDoc, SyncPeer};

// ── WsTransport ───────────────────────────────────────────────────────────────

/// A WebSocket connection used as a binary transport for Automerge sync messages.
///
/// Connect with [`WsTransport::connect`] or accept an incoming connection with
/// [`WsTransport::from_stream`].
pub struct WsTransport {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsTransport {
    /// Connect to a WebSocket server at `url` (e.g. `"ws://peer.example.com/sync"`).
    pub async fn connect(url: &str) -> Result<Self, FsnError> {
        debug!(url, "WsTransport: connecting");
        let (ws, _response) = connect_async(url).await.map_err(|e| {
            FsnError::network(format!("WebSocket connect failed ({url}): {e}"))
        })?;
        debug!(url, "WsTransport: connected");
        Ok(Self { ws })
    }

    /// Wrap an already-accepted incoming WebSocket stream.
    pub fn from_stream(ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { ws }
    }

    /// Send a raw sync message binary frame.
    pub async fn send(&mut self, bytes: Vec<u8>) -> Result<(), FsnError> {
        self.ws
            .send(Message::Binary(bytes.into()))
            .await
            .map_err(|e| FsnError::network(format!("WsTransport send failed: {e}")))
    }

    /// Receive the next binary frame. Returns `None` on clean close.
    pub async fn recv(&mut self) -> Result<Option<Vec<u8>>, FsnError> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Binary(data))) => return Ok(Some(data.into())),
                Some(Ok(Message::Ping(data))) => {
                    // Auto-respond to pings
                    let _ = self.ws.send(Message::Pong(data)).await;
                }
                Some(Ok(Message::Close(_))) | None => return Ok(None),
                Some(Ok(_)) => {} // ignore text / pong / other frames
                Some(Err(e)) => {
                    return Err(FsnError::network(format!("WsTransport recv failed: {e}")))
                }
            }
        }
    }

    /// Send a WebSocket Close frame.
    pub async fn close(&mut self) {
        let _ = self.ws.send(Message::Close(None)).await;
    }
}

// ── SyncSession<T> ────────────────────────────────────────────────────────────

/// A live sync session that exchanges Automerge messages over a [`WsTransport`].
///
/// Create via [`SyncSession::new`], then drive with:
/// - [`run_until_synced`] — exchange messages until both peers converge
/// - [`push`] — generate and send one sync message
/// - [`pull`] — receive and apply one sync message
///
/// # Example
///
/// ```rust,ignore
/// use fsn_sync::{SyncEngine, SyncSession, WsTransport};
///
/// let engine = SyncEngine::new("node-a".into());
/// let mut doc = engine.new_doc::<Config>();
/// doc.set(&my_config)?;
///
/// let transport = WsTransport::connect("ws://peer:8080/sync").await?;
/// let peer = engine.new_peer("node-b");
///
/// let mut session = SyncSession::new(doc, peer, transport);
/// session.run_until_synced().await?;
///
/// // After sync, both sides have converged
/// let merged: Config = session.into_doc().get()?;
/// ```
pub struct SyncSession<T: Serialize + for<'de> Deserialize<'de>> {
    doc: SyncDoc<T>,
    peer: SyncPeer,
    transport: WsTransport,
}

impl<T: Serialize + for<'de> Deserialize<'de>> SyncSession<T> {
    /// Create a new session from an existing doc, peer tracker, and transport.
    pub fn new(doc: SyncDoc<T>, peer: SyncPeer, transport: WsTransport) -> Self {
        Self { doc, peer, transport }
    }

    /// Exchange sync messages until both sides have converged.
    ///
    /// The algorithm:
    /// 1. Generate a local sync message and send it (if any).
    /// 2. Receive one message from the remote side.
    /// 3. Apply it to the local doc.
    /// 4. Repeat until both sides produce `None` (= no more changes to send).
    ///
    /// After this returns `Ok(())`, `doc.get()` returns the merged value.
    pub async fn run_until_synced(&mut self) -> Result<(), FsnError> {
        loop {
            // Send local changes (if any)
            let local_msg = self.doc.generate_sync_message(&mut self.peer);

            if let Some(bytes) = local_msg {
                debug!(bytes = bytes.len(), "SyncSession: sending sync message");
                self.transport.send(bytes).await?;
            }

            // Receive remote message
            match self.transport.recv().await? {
                Some(bytes) => {
                    debug!(bytes = bytes.len(), "SyncSession: received sync message");
                    self.doc.receive_sync_message(&mut self.peer, &bytes)?;
                }
                None => {
                    // Remote closed — check if we're done
                    debug!("SyncSession: remote closed, checking convergence");
                    break;
                }
            }

            // Check if we've converged: no more local messages to send
            let check = self.doc.generate_sync_message(&mut self.peer);
            if check.is_none() {
                // We have nothing left to send — close and finish
                debug!("SyncSession: converged, closing");
                self.transport.close().await;
                break;
            }
        }

        Ok(())
    }

    /// Generate and send one sync message to the remote peer.
    ///
    /// Returns `true` if a message was sent, `false` if there is nothing to send.
    pub async fn push(&mut self) -> Result<bool, FsnError> {
        match self.doc.generate_sync_message(&mut self.peer) {
            Some(bytes) => {
                self.transport.send(bytes).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Receive one sync message from the remote peer and apply it.
    ///
    /// Returns `true` if a message was received, `false` if the connection closed.
    pub async fn pull(&mut self) -> Result<bool, FsnError> {
        match self.transport.recv().await? {
            Some(bytes) => {
                self.doc.receive_sync_message(&mut self.peer, &bytes)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Consume the session and return the underlying `SyncDoc`.
    pub fn into_doc(self) -> SyncDoc<T> {
        self.doc
    }

    /// Borrow the underlying `SyncDoc`.
    pub fn doc(&self) -> &SyncDoc<T> {
        &self.doc
    }

    /// Mutably borrow the underlying `SyncDoc`.
    pub fn doc_mut(&mut self) -> &mut SyncDoc<T> {
        &mut self.doc
    }
}
