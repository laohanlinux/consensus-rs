//! Core message handle - replaces Addr<Core> for sending messages to consensus core

use crossbeam::channel::Sender;

use crate::consensus::events::{MessageEvent, NewHeaderEvent, FinalCommittedEvent, BackLogEvent, TimerEvent, OpCMD};
use crate::consensus::types::Proposal;

/// Messages that can be sent to the Core
#[derive(Debug)]
pub enum CoreMessage {
    Message(MessageEvent),
    NewHeader(NewHeaderEvent),
    FinalCommitted(FinalCommittedEvent),
    BackLog(BackLogEvent),
    Timer(TimerEvent),
    Op(OpCMD),
}

/// Handle for sending messages to the Core (replaces Addr<Core>)
#[derive(Clone)]
pub struct CoreHandle {
    tx: Sender<CoreMessage>,
}

impl CoreHandle {
    pub fn new(tx: Sender<CoreMessage>) -> Self {
        Self { tx }
    }

    pub fn send_message(&self, payload: Vec<u8>) {
        let _ = self.tx.try_send(CoreMessage::Message(MessageEvent { payload }));
    }

    pub fn send_new_header(&self, proposal: Proposal) {
        let _ = self.tx.try_send(CoreMessage::NewHeader(NewHeaderEvent { proposal }));
    }

    pub fn send_final_committed(&self) {
        let _ = self.tx.try_send(CoreMessage::FinalCommitted(FinalCommittedEvent {}));
    }

    pub fn send_backlog(&self, msg: crate::protocol::GossipMessage) {
        let _ = self.tx.try_send(CoreMessage::BackLog(BackLogEvent { msg }));
    }

    pub fn send_timer(&self) {
        let _ = self.tx.try_send(CoreMessage::Timer(TimerEvent {}));
    }

    pub fn send_stop(&self) {
        let _ = self.tx.try_send(CoreMessage::Op(OpCMD::Stop));
    }
}
