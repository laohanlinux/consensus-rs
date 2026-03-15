//! Event types and buses (replaces actix-broker)

use libp2p::PeerId;

use crate::types::block::{Block, Blocks, Header};
use crate::types::Height;

pub const MAX_MAILBOX_CAPACITY: usize = 1 << 11;

/// Chain events
#[derive(Clone, Debug)]
pub enum ChainEvent {
    NewBlock(Block),
    NewHeader(Header),
    SyncBlock(Height),
    PostBlock(Option<PeerId>, Blocks),
}

/// Chain event bus - replaces ProcessSignals for ChainEvent
#[derive(Clone)]
pub struct ChainEventBus {
    tx: tokio::sync::broadcast::Sender<ChainEvent>,
}

impl ChainEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, event: ChainEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ChainEvent> {
        self.tx.subscribe()
    }
}

use crate::types::transaction::Transaction;
use crate::protocol::GossipMessage;

/// Broadcast events (consensus, blocks, sync)
#[derive(Clone, Debug)]
pub enum BroadcastEvent {
    Transaction(Transaction),
    Blocks(Option<PeerId>, Blocks),
    Consensus(GossipMessage),
    Sync(Height),
}

/// Broadcast event bus - replaces BroadcastEventSubscriber
#[derive(Clone)]
pub struct BroadcastEventBus {
    tx: tokio::sync::broadcast::Sender<BroadcastEvent>,
}

impl BroadcastEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, event: BroadcastEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BroadcastEvent> {
        self.tx.subscribe()
    }
}
