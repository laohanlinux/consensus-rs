use crate::{
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::types::{Subject, View, Request},
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    consensus::events::{RequestEvent, NewHeaderEvent},
    types::Validator,
};

use super::{
    core::Core,
    commit::HandleCommit,
    request::HandlerRequest,
};

pub trait HandleNewHeader {
    fn handle(&mut self, msg: &NewHeaderEvent, src: &Validator) -> ConsensusResult;
}

impl HandleNewHeader for Core {
    fn handle(&mut self, msg: &NewHeaderEvent, src: &Validator) -> ConsensusResult {
        // start new round, height = last_height + 1
        self.start_new_zero_round();
        <Core as HandlerRequest>::handle(self, &Request::new(msg.proposal.clone()))
    }
}