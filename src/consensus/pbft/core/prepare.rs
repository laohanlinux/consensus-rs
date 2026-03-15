use std::borrow::Cow;

use cryptocurrency_kit::storage::values::StorageValue;


use crate::{
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::types::Subject,
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    types::Validator,
};

use super::{
    core::CoreState,
    commit::HandleCommit,
};

pub trait HandlePrepare {
    fn send_prepare(&mut self);
    fn verify_prepare(&mut self, prepare: &Subject, src: &Validator) -> ConsensusResult;
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult;
    fn accept(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult;
}

impl HandlePrepare for CoreState {
    fn send_prepare(&mut self) {
        let subject = self.current_state.subject().as_ref().cloned().unwrap();
        let payload = subject.into_bytes();
        self.broadcast(&GossipMessage::new(MessageType::Prepare, payload, None));
    }

    fn verify_prepare(&mut self, subject: &Subject, _src: &Validator) -> ConsensusResult {
        let current_view = self.current_view();
        if current_view != subject.view {
            return Err(ConsensusError::InconsistentSubject);
        }
        Ok(())
    }

    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        let subject: Subject = Subject::from_bytes(Cow::from(msg.msg()));
        self.check_message(MessageType::Prepare, &subject.view)?;
        self.verify_prepare(&subject, src)?;
        <CoreState as HandlePrepare>::accept(self, msg, src)?;
        if self.current_state.is_locked()
            && subject.digest == *self.current_state.get_lock_hash().as_ref().unwrap()
        {
            self.current_state.lock_hash();
            self.set_state(State::Prepared);
            self.send_commit();
        }
        if self.current_state.get_prepare_or_commit_size() > self.val_set().two_thirds_majority() {
            self.current_state.lock_hash();
            self.set_state(State::Prepared);
            self.send_commit();
        }
        Ok(())
    }

    fn accept(&mut self, msg: &GossipMessage, _src: &Validator) -> ConsensusResult {
        self.current_state
            .prepares
            .add(msg.clone())
            .map_err(ConsensusError::Unknown)
    }
}
