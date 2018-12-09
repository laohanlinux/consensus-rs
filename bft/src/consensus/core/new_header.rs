use crate::{
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::types::{Subject, View},
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    consensus::events::NewHeaderEvent,
    types::Validator,
};

use super::{
    core::Core,
    commit::HandleCommit,
};

pub trait HandleNewHeader {
    fn handle(&mut self, msg: &NewHeaderEvent, src: &Validator) -> ConsensusResult;
}

impl HandleNewHeader for Core {
    fn handle(&mut self, msg: &NewHeaderEvent, src: &Validator) -> ConsensusResult {
        Ok(())
    }
}