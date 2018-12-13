use std::borrow::Cow;
use std::time::Duration;

use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::{CryptoHash, Hash, hash};

use crate::{
    consensus::error::{ConsensusError, ConsensusResult, EngineError},
    consensus::types::{PrePrepare, Proposal, Request, Subject},
    consensus::validator::Validators,
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    types::Validator,
};

use super::{
    round_change::HandleRoundChange,
    core::Core,
    commit::HandleCommit,
    prepare::HandlePrepare,
};

pub trait HandlePreprepare {
    fn send_preprepare(&mut self, requst: &Request<Proposal>);
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> Result<(), ConsensusError>;
    fn accetp(&mut self, preprepare: &PrePrepare);
}

impl HandlePreprepare for Core {
    fn send_preprepare(&mut self, request: &Request<Proposal>) {
        //TODO add lock hash prove
        if self.current_state.height() == request.proposal().block().height() && self.is_proposer()
            {
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
        // Ensure we have the same view with the PRE-PREPARE message
        // If it is old message, see if we need to broadcast COMMIT
        if let Err(ref err) = result {
            match err {
                ConsensusError::OldMessage => {
                    let block = preprepare.proposal.block();
                    let pre_header = match self.backend.get_header_by_height(block.height()) {
                        Some(header) => {
                            header
                        }
                        None => {
                            return Err(ConsensusError::Engine(EngineError::InvalidProposal));
                        }
                    };
                    if pre_header.block_hash() != block.hash() {
                        return Err(ConsensusError::Engine(EngineError::InvalidProposal));
                    }
                    let pre_height = block.height() - 1;
                    let mut val_set = self.backend.validators(pre_height).clone();
                    let _previous_proposer = self.backend.get_proposer(pre_height);
                    val_set.calc_proposer(&block.header().prev_hash, pre_height, preprepare.view.round);
                    if val_set.is_proposer(src.address().clone())
                        && self.backend.has_proposal(&block.hash(), block.height())
                        {
                            self.send_commit_for_old_block(&preprepare.view, block.hash());
                        }
                }
                ConsensusError::FutureBlockMessage(_) => {
                    // forward EngineError::FutureBlock to handle
                    // self.new_round_future_preprepare_timer()
                }
                _ => return result
            }
        }

        let val_set = self.val_set();
        if val_set.is_proposer(src.address().clone()) == false {
            return Err(ConsensusError::NotFromProposer);
        }

        // TODO
        let (d, result) = self
            .backend
            .verify(&preprepare.proposal);

        if let Err(ref err) = result {
            match err {
                EngineError::FutureBlock => {
                    self.new_round_future_preprepare_timer(d, msg.clone());
                    return Err(ConsensusError::FutureBlockMessage(preprepare.proposal.block().height()));
                }
                // other error
                _ => {
                    // send next round change, because proposal is invalid, so proposer is bad node
                    self.send_next_round_change();
                    return Err(ConsensusError::Unknown(format!("{}", err)));
                }
            }
        }

        if self.state == State::AcceptRequest {
            if self.current_state.is_locked() {
                if preprepare.proposal.block().hash() == self.current_state.get_lock_hash().unwrap() {
                    <Core as HandlePreprepare>::accetp(self, &preprepare);
                    self.set_state(State::Prepared);
                    self.send_commit();
                } else {
                    self.send_next_round_change();
                }
            } else {
                <Core as HandlePreprepare>::accetp(self, &preprepare);
                self.set_state(State::Preprepared);
                self.send_prepare();
            }
        }

        // TODO
        Ok(())
    }

    fn accetp(&mut self, preprepare: &PrePrepare) {
        let header = preprepare.proposal.block().header();
        self.consensus_timestamp = Duration::from_nanos(header.time);
        self.current_state.set_preprepare(preprepare.clone())
    }
}
