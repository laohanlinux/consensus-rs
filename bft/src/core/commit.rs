
use cryptocurrency_kit::crypto::{Hash};
use cryptocurrency_kit::ethkey::Address;

use crate::{
    protocol::{MessageType, GossipMessage},
    types::Validator,
    consensus::types::{View, Subject},
};

pub trait Commit {
    fn handle_commit(&mut self, msg: &GossipMessage, src: Validator) -> Result<(), String>;
    fn send_commit(&mut self);
    fn send_commit_for_old_block(&mut self, view: &View, digest: Hash);
    fn broadcast_commit(&mut self, sub: &Subject, seal: Hash);
}