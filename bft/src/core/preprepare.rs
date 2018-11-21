
use crate::{
    types::Validator,
    protocol::{GossipMessage},
    consensus::validator::{Validators},
    consensus::types::{Request, Proposal},
};

use super::core::Core;

pub trait HandlePreprepare {
    fn send_preprepare(&self, requst: &Request<Proposal>);
    fn handle(&self, msg: &GossipMessage, src: Validator) -> Result<(), String>;
}

impl HandlePreprepare for Core {
    fn send_preprepare(&self, request: &Request<Proposal>) {

    }

    fn handle(&self, msg: &GossipMessage, src: Validator) -> Result<(), String> {
        Ok(())
    }
}