use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::{
    public_to_address, recover, verify_address, Address, Message as SignMessage, Signature,
};
use cryptocurrency_kit::storage::values::StorageValue;

use super::core::Core;
use crate::{
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::types::{Subject, View},
    consensus::validator::ValidatorSet,
    protocol::{GossipMessage, MessageType, State},
    types::{
        votes::{decrypt_commit_bytes, encrypt_commit_bytes, Votes},
        Validator,
    },
};

use std::borrow::Cow;
use ethereum_types::H256;
use cryptocurrency_kit::common::to_fixed_array_32;

pub trait HandleCommit {
    fn send_commit(&mut self);
    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash);
    fn broadcast_commit(&mut self, sub: &Subject, seal: Hash);
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> Result<(), ConsensusError>;
    fn verify_commit(
        &self,
        commit_seal: Option<&Signature>,
        subject: &Subject,
        sender: Address,
        src: Validator,
    ) -> Result<(), ConsensusError>;
    fn accept(&mut self, msg: &GossipMessage, src: &Validator) -> Result<(), ConsensusError>;
}

impl HandleCommit for Core {
    fn send_commit(&mut self) {
        let current_state = &self.current_state;
        let proposal = current_state.proposal().unwrap();
        let block = proposal.block();
        let subject = current_state.subject();
        self.broadcast_commit(subject.as_ref().unwrap(), block.hash())
    }

    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash) {
        let subject = Subject {
            view: view.clone(),
            digest: digest,
        };
        self.broadcast_commit(&subject, digest)
    }

    // TOOD
    fn broadcast_commit(&mut self, subject: &Subject, _digest: Hash) {
        trace!("broadcast commit");
        let commit_seal = encrypt_commit_bytes(&subject.digest, self.keypair.secret());
        let encoded_subject = subject.clone().into_bytes();
        let msg = GossipMessage::new(MessageType::Commit, encoded_subject, Some(commit_seal));
        self.broadcast(&msg);
    }

    // handle commit type message
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> Result<(), ConsensusError> {
        debug!("Handle commit message from {:?}", src.address());
        let subject = Subject::from(msg.msg());
        //        let _current_subject = self.current_state.subject().unwrap();
        self.check_message(MessageType::Commit, &subject.view)?;
        let sender = msg.address;
        let subject = Subject::from_bytes(Cow::from(msg.msg()));
        self.verify_commit(msg.commit_seal.as_ref(), &subject, sender, src.clone())?;
        debug!(
            "Pass very commit, commit size:{}, state:{:?}, {}",
            self.current_state.commits.len(),
            self.state,
            msg.trace()
        );
        <Core as HandleCommit>::accept(self, msg, src)?;
        let val_set = self.val_set();
        // receive more +2/3 votes
        if self.current_state.commits.len() > val_set.two_thirds_majority()
            && self.state < State::Committed
        {
            self.current_state.lock_hash();
            self.commit();
        }
        Ok(())
    }

    fn verify_commit(
        &self,
        commit_seal: Option<&Signature>,
        commit_subject: &Subject,
        sender: Address,
        _src: Validator,
    ) -> Result<(), ConsensusError> {
        if commit_seal.is_none() {
            return Err(ConsensusError::Unknown("commit seal is nil".to_string()));
        }
        let commit_seal = commit_seal.unwrap();
        let digest = H256::from(to_fixed_array_32(commit_subject.digest.as_ref()));
        let sign_message = SignMessage::from(digest);
        verify_address(&sender, commit_seal, &sign_message)
            .map(|_| ())
            .map_err(|_| {
                ConsensusError::Unknown("message's sender should be commit seal".to_string())
            })?;
        let current_state = &self.current_state;
        let current_subject = current_state.subject().unwrap();
        if current_subject.digest != commit_subject.digest
            || current_subject.view != commit_subject.view
        {
            warn!(
                "Inconsistent subjects between commit and proposal, d1={}, d2={}",
                current_subject.digest.short(),
                commit_subject.digest.short()
            );
            //return Err(ConsensusError::Unknown(
            //    "Inconsistent subjects between commit and proposal".to_string(),
            //));
        }
        Ok(())
    }

    fn accept(&mut self, msg: &GossipMessage, _: &Validator) -> ConsensusResult {
        self.current_state
            .commits
            .add(msg.clone())
            .map_err(|err| ConsensusError::Unknown(err))
    }
}
