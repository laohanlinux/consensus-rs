use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::Address;

use std::time::Duration;

use super::types::{Height, Proposal};
use super::validator::{Validator, ValidatorSet};

pub trait Backend {
    /// address is the current validator's address
    fn address(&self) -> Address;
    /// validators returns a set of current validator
    fn validators(&self) -> &ValidatorSet;
    /// TODO
    fn event_mux(&self);
    /// broadcast sends a message to all validators (include itself)
    fn broadcast(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// gossip sends a message to all validators (exclude self)
    fn gossip(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// commit a proposal with seals
    fn commit(&mut self, proposal: &Proposal, seals: &[&[u8]]) -> Result<(), ()>;
    /// verifies the proposal. If a err_future_block error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), ()>);
    fn sign(&self, digest: &[u8]) -> (Vec<u8>, Result<(), ()>);
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<(), ()>;

    fn last_proposal(&self) -> (&Proposal, Result<(), ()>);
    fn has_proposal(&self, hash: Hash, height: Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators(&self, proposal: &Proposal) -> &ValidatorSet;
    fn has_bad_proposal(hash: Hash) -> bool;
}
