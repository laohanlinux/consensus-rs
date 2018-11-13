use lru_time_cache::LruCache;

use cryptocurrency_kit::crypto::hash;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::keccak::Keccak256;
use cryptocurrency_kit::ethkey::{
    sign, verify_address, Address, KeyPair, Message, Public, Secret, Signature,
};
use std::time::Duration;
use std::sync::{Arc, RwLock};

use store::ledger::Ledger;
use super::types::Proposal;
use super::validator::{ImplValidatorSet, ValidatorSet};
use types::{Height, Validator, EMPTY_ADDRESS};

pub trait Backend {
    /// address is the current validator's address
    fn address(&self) -> Address;
    /// validators returns a set of current validator
    fn validators(&self) -> &ValidatorSet;
    ///TODO
    fn event_mux(&self);
    /// broadcast sends a message to all validators (include itself)
    fn broadcast(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// gossip sends a message to all validators (exclude self)
    fn gossip(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// commit a proposal with seals
    fn commit(&mut self, proposal: &Proposal, seals: &[&[u8]]) -> Result<(), ()>;
    /// verifies the proposal. If a err_future_block error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> Result<Duration, ()>;
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()>;
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()>;

    fn last_proposal(&self) -> Result<&Proposal, ()>;
    fn has_proposal(&self, hash: &Hash, height: &Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators(&self, proposal: &Proposal) -> &ValidatorSet;
    fn has_bad_proposal(hash: Hash) -> bool;
}

struct ImplBackend<T: ValidatorSet> {
    validaor: Validator,
    validator_set: T,
    key_pair: KeyPair,
    inbound_cache: LruCache<Hash, String>,
    outbound_cache: LruCache<Hash, String>,
    ledger: Arc<RwLock<Ledger>>,
}

impl<T> Backend for ImplBackend<T>
where
    T: ValidatorSet,
{
    fn address(&self) -> Address {
        *self.validaor.address()
    }

    fn validators(&self) -> &ValidatorSet {
        &self.validator_set
    }

    /// TODO
    fn event_mux(&self) {}

    /// TODO
    fn broadcast(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()> {
        Err(())
    }

    /// TODO
    fn gossip(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()> {
        Err(())
    }

    /// TODO
    fn commit(&mut self, proposal: &Proposal, seals: &[&[u8]]) -> Result<(), ()> {
        Err(())
    }

    /// TODO
    fn verify(&self, proposal: &Proposal) -> Result<Duration, ()> {
        Err(())
    }

    /// TODO
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, ()> {
        let message = Message::from(digest);
        match sign(&self.key_pair.secret(), &message) {
            Ok(signature) => Ok(signature.to_vec()),
            Err(_) => Err(()),
        }
    }

    /// TODO
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()> {
        let keccak_hash = hash(data);
        let signature = Signature::from_slice(sig);
        verify_address(&address, &signature, &Message::from(keccak_hash.as_ref())).map_err(|_| ())
    }

    /// TODO
    fn last_proposal(&self) -> Result<&Proposal, ()> {
        let ledger = self.ledger.read().unwrap();
        let block = ledger.get_last_block();
        Err(())
    }

    /// TODO
    fn has_proposal(&self, hash: &Hash, height: &Height) -> bool {
        false
    }

    /// TODO
    fn get_proposer(&self, height: Height) -> Address {
        Address::from(0)
    }

    /// TODO
    fn parent_validators(&self, proposal: &Proposal) -> &ValidatorSet {
        &self.validator_set
    }

    /// TODO
    fn has_bad_proposal(hash: Hash) -> bool {
        false
    }
}

impl<T> ImplBackend<T> where T: ValidatorSet {}
