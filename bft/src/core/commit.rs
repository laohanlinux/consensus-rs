use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::{Address, Signature, Message as SignMessage, public_to_address, verify_address, recover};

use crate::{
    protocol::{MessageType, GossipMessage},
    types::{Validator, votes::Votes},
    consensus::types::{View, Subject},
};
use super::{core::Core};
use super::types::State;


pub trait Commit {
    fn send_commit(&mut self);
    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash);
    fn broadcast_commit(&mut self, sub: &Subject, seal: Hash);
    fn handle(&mut self, msg: &GossipMessage, src: Validator) -> Result<(), String>;
    fn verify_commit(&self, commit_seal: Option<&Signature>, subject: &Subject, sender: Address, src: Validator) -> Result<(), String>;
    fn accept(&mut self, msg: GossipMessage, src: Validator) -> Result<(), String>;
}


impl Commit for Core {
    // TODO
    fn send_commit(&mut self) {

    }

    // TODO
    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash) {

    }
    // TOOD
    fn broadcast_commit(&mut self, subject: &Subject, digest: Hash) {

    }

    fn handle(&mut self, msg: &GossipMessage, src: Validator) -> Result<(), String> {
        let subject = Subject::from(msg.msg());
        let current_state = self.mut_current_state();
        let current_subject = current_state.subject().unwrap();
        self.check_message(MessageType::Commit, &subject.view)?;
        match msg.address() {
            Ok(sender) => {
                self.verify_commit(msg.commit_seal.as_ref(), sender, src)?;
                self.accept(msg.clone(), src)?;
                let current_state = self.mut_current_state();
                let val_set = self.val_set();
                let subject = Subject::from_bytes(msg.msg());
                if current_state.commits.len() > val_set().two_thirds_majority() && self.state < State::Committed {
                    current_state.lock_hash();
                    self.commit();
                }
            }
            Err(reason) => {
                return Err(reason);
            }
        }
        Err("".to_string())
    }

    fn verify_commit(&self, commit_seal: Option<&Signature>, commit_subject: &Subject, sender: Address, src: Validator) -> Result<(), String> {
        if commit_seal.is_none() {
            return Err("commit seal is nil".to_string());
        }
        let commit_seal = commit_seal.unwrap();
        let sign_message = SignMessage::from(subject.digest.as_ref());
        verify_address(&sender, commit_seal, &sign_message).map(|_| ()).map_err(|_| "message's sender should be commit seal".to_string())?;
        let current_state = self.mut_current_state();
        let current_subject = current_state.subject().unwrap();
        if current_subject != commit_subject {
            Err("Inconsistent subjects between commit and proposal".to_string());
        }
        Ok(())
    }

    fn accept(&mut self, msg: GossipMessage, src: Validator) -> Result<(), String> {
        let current_state = self.mut_current_state();
        current_state.commits.add(msg.clone())
    }
}
