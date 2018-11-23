use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::{Address, Signature, Message as SignMessage, public_to_address, verify_address, recover};
use cryptocurrency_kit::storage::values::StorageValue;

use crate::{
    protocol::{MessageType, GossipMessage, State},
    types::{Validator, votes::Votes},
    consensus::types::{View, Subject},
    consensus::validator::ValidatorSet,
};
use super::{core::Core};

use std::borrow::Cow;


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
        let current_state = self.mut_current_state();
        let subject = current_state.subject();
        self.broadcast_commit(subject.as_ref().unwrap(), current_state.proposal().unwrap().block().hash())
    }

    // TODO
    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash) {
        let subject = Subject {
            view: view.clone(),
            digest: digest,
        };
        self.broadcast_commit(&subject, digest)
    }
    // TOOD
    fn broadcast_commit(&mut self, subject: &Subject, digest: Hash) {
        trace!("broadcast commit");
        // commit sign
        let commit_seal = digest.sign(&self.keypair.secret()).unwrap();
        let encoded_subject = subject.clone().into_bytes();
        let mut msg = GossipMessage::new(MessageType::Commit, encoded_subject, Some(commit_seal));
        // TODO add extra
        // message sign
        msg.set_sign(self.keypair.secret());
        self.broadcast(&msg);
    }


    // handle commit type message
    fn handle(&mut self, msg: &GossipMessage, src: Validator) -> Result<(), String> {
        let subject = Subject::from(msg.msg());
        let current_state = self.mut_current_state();
        let current_subject = current_state.subject().unwrap();
        self.check_message(MessageType::Commit, &subject.view)?;
        match msg.address() {
            Ok(sender) => {
                let subject = Subject::from_bytes(Cow::from(msg.msg()));
                self.verify_commit(msg.commit_seal.as_ref(), &subject, sender, src)?;
                self.accept(msg.clone(), src)?;
                let current_state = self.mut_current_state();
                let val_set = self.val_set();
                if current_state.commits.len() > val_set.two_thirds_majority() && self.state < State::Committed {
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
        let sign_message = SignMessage::from(commit_subject.digest.as_ref());
        verify_address(&sender, commit_seal, &sign_message).map(|_| ()).map_err(|_| "message's sender should be commit seal".to_string())?;
        let current_state = self.mut_current_state();
        let current_subject = current_state.subject().unwrap();
        if current_subject.digest != commit_subject.digest || current_subject.view != commit_subject.view  {
            return Err("Inconsistent subjects between commit and proposal".to_string());
        }
        Ok(())
    }

    fn accept(&mut self, msg: GossipMessage, src: Validator) -> Result<(), String> {
        let current_state = self.mut_current_state();
        current_state.commits.add(msg.clone())
    }
}
