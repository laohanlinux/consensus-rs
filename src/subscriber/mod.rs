//! Event buses using tokio broadcast channels (replaces actix ProcessSignals)

use libp2p::Multiaddr;
use libp2p::PeerId;
use tokio::sync::broadcast;

pub mod events;


/// P2P discovery events
#[derive(Clone, Debug)]
pub enum P2PEvent {
    AddPeer(PeerId, Vec<Multiaddr>),
    DropPeer(PeerId, Vec<Multiaddr>),
}

/// P2P event bus - replaces ProcessSignals for P2PEvent
#[derive(Clone)]
pub struct P2PEventBus {
    tx: broadcast::Sender<P2PEvent>,
}

impl P2PEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, event: P2PEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<P2PEvent> {
        self.tx.subscribe()
    }
}

pub fn spawn_sync_subscriber() -> P2PEventBus {
    P2PEventBus::new(64)
}
