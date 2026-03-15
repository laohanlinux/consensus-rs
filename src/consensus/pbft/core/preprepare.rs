use std::borrow::Cow;
use std::time::Duration;

use cryptocurrency_kit::storage::values::StorageValue;

use crate::{
    consensus::error::{ConsensusError, ConsensusResult, EngineError},
    consensus::types::{PrePrepare, Proposal, Request},
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    types::Validator,
};

use super::{
    round_change::HandleRoundChange,
    core::CoreState,
    commit::HandleCommit,
    prepare::HandlePrepare,
};

pub trait HandlePreprepare {
    fn send_preprepare(&mut self, requst: &Request<Proposal>);
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> Result<(), ConsensusError>;
    fn accetp(&mut self, preprepare: &PrePrepare);
}

impl HandlePreprepare for CoreState {
    fn send_preprepare(&mut self, request: &Request<Proposal>) {
        if self.current_state.height() == request.proposal().block().height() && self.is_proposer() {
            let preprepre = PrePrepare::new(self.current_view(), request.proposal.clone());
            self.broadcast(&GossipMessage::new(
                MessageType::Preprepare,
                preprepre.into_bytes(),
                None,
            ));
        } else {
            debug!("Im's not proposer");
        }
    }

    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        let preprepare: PrePrepare = PrePrepare::from_bytes(Cow::from(msg.msg()));
        let result = self.check_message(MessageType::Preprepare, &preprepare.view);
        if let Err(ref err) = result {
            match err {
                ConsensusError::OldMessage => {
                    let block = preprepare.proposal.block();
                    let pre_header = self.backend.get_header_by_height(block.height())
                        .ok_or(ConsensusError::Engine(EngineError::InvalidProposal))?;
                    if pre_header.block_hash() != block.hash() {
                        return Err(ConsensusError::Engine(EngineError::InvalidProposal));
                    }
                    let pre_height = block.height() - 1;
                    let mut val_set = self.backend.validators(pre_height).clone();
                    let _ = self.backend.get_proposer(pre_height);
                    val_set.calc_proposer(&block.header().prev_hash, pre_height, preprepare.view.round);
                    if val_set.is_proposer(*src.address())
                        && self.backend.has_proposal(&block.hash(), block.height())
                    {
                        self.send_commit_for_old_block(&preprepare.view, block.hash());
                    }
                    return result;
                }
                ConsensusError::FutureBlockMessage(_) => {}
                _ => return result,
            }
        }

        if !self.val_set().is_proposer(*src.address()) {
            return Err(ConsensusError::NotFromProposer);
        }

        let (d, result) = self.backend.verify(&preprepare.proposal);
        if let Err(ref err) = result {
            match err {
                EngineError::FutureBlock => {
                    self.new_round_future_preprepare_timer(d, msg.clone());
                    return Err(ConsensusError::FutureBlockMessage(preprepare.proposal.block().height()));
                }
                _ => {
                    self.send_next_round_change();
                    return Err(ConsensusError::Unknown(format!("{}", err)));
                }
            }
        }

        if self.state == State::AcceptRequest {
            if self.current_state.is_locked() {
                if preprepare.proposal.block().hash() == self.current_state.get_lock_hash().unwrap() {
                    <CoreState as HandlePreprepare>::accetp(self, &preprepare);
                    self.set_state(State::Prepared);
                    self.send_commit();
                } else {
                    self.send_next_round_change();
                }
            } else {
                <CoreState as HandlePreprepare>::accetp(self, &preprepare);
                self.set_state(State::PrePrepared);
                self.send_prepare();
            }
        }

        Ok(())
    }

    fn accetp(&mut self, preprepare: &PrePrepare) {
        let header = preprepare.proposal.block().header();
        self.consensus_timestamp = Duration::from_nanos(header.time);
        self.current_state.set_preprepare(preprepare.clone())
    }
}
