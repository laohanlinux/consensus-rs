use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::Hash;

use std::time::Duration;

use super::validator::ValidatorSet;
use super::types::{Height, Proposal};

pub trait Backend {
    fn address(&self) -> Address;
    fn validators<T: ValidatorSet>(&self) -> &T;

    // TODO
    fn event_mux(&self);

    /// Broadcast sends a message to all validators (include self)
    fn broadcast<T: ValidatorSet>(&self, vals: &T, payload: &[u8]) -> Result<(), ()>;
    /// Gossip sends a message to all validators (exclude self)
    fn gossip<T: ValidatorSet>(&self, vals: &T, payload: &[u8]) -> Result<(), ()>;

    fn commit(&mut self, proposal: &Proposal, seals: &[&[u8]]) -> Result<(), ()>;

    /// Verify verifies the proposal. If a consensus.ErrFutureBlock error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), ()>);
    fn sign(&self, digest: &[u8]) -> (Vec<u8>, Result<(), ()>);
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<(), ()>;

    fn last_proposal(&self) -> (&Proposal, Result<(), ()>);
    fn has_proposal(&self, hash: Hash, height: Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators<T: ValidatorSet>(&self, proposal: &Proposal) -> &T;
    fn has_bad_proposal(hash: Hash) -> bool;
}