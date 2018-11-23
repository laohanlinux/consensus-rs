use std::borrow::Cow;

use cryptocurrency_kit::storage::values::StorageValue;

use crate::{
    types::Validator,
    protocol::{GossipMessage, MessageType},
    consensus::validator::Validators,
    consensus::types::{Request, Proposal, PrePrepare},
};

use super::core::Core;

pub trait HandlePreprepare {
    fn send_preprepare(&self, requst: &Request<Proposal>);
    fn handle(&self, msg: &GossipMessage, src: Validator) -> Result<(), String>;
}

impl HandlePreprepare for Core {
    fn send_preprepare(&self, request: &Request<Proposal>) {
        //TODO add lock hash prove
        if self.current_state.height() == request.proposal().block().height() && self.is_proposer() {
            let mut preprepre = PrePrepare::new(self.current_view(), request.proposal.clone());
            self.broadcast(&GossipMessage::new(MessageType::Preprepare, preprepre.into_bytes(), None));
        }
    }

    fn handle(&self, msg: &GossipMessage, src: Validator) -> Result<(), String> {
        let mut preprepare = PrePrepare::from_bytes(Cow::from(msg.msg()));
        self.check_message(MessageType::Preprepare, &preprepare.view)?;

        // TODO
        Ok(())
    }
}