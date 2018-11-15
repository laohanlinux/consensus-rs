use lru_time_cache::LruCache;

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash, EMPTY_HASH};
use cryptocurrency_kit::ethkey::keccak::Keccak256;
use cryptocurrency_kit::ethkey::{
    sign, verify_address, Address, KeyPair, Message, Public, Secret, Signature,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::config::Config;
use super::types::Proposal;
use super::validator::{ImplValidatorSet, ValidatorSet};
use common::merkle_tree_root;
use store::ledger::Ledger;
use types::block::Header;
use types::transaction::Transaction;
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
    fn verify(&self, proposal: &Proposal) -> Result<(), String>;
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, String>;
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()>;

    fn last_proposal(&self) -> Result<Proposal, ()>;
    fn has_proposal(&self, hash: &Hash, height: Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators(&self, proposal: &Proposal) -> &ValidatorSet;
    fn has_bad_proposal(&self, hash: Hash) -> bool;
}

struct ImplBackend<T: ValidatorSet> {
    validaor: Validator,
    validator_set: T,
    key_pair: KeyPair,
    inbound_cache: LruCache<Hash, String>,
    outbound_cache: LruCache<Hash, String>,
    ledger: Arc<RwLock<Ledger>>,
    config: Config,
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
    fn verify(&self, proposal: &Proposal) -> Result<(), String> {
        let block = &proposal.0;
        let header = block.header();
        let blh = header.hash();
        if self.has_bad_proposal(blh) {
            return Err("bad unit".to_string());
        }

        // check transaction
        {
            let transactions = block.transactions().to_vec();
            for transaction in &transactions {
                if !transaction.verify_sign(self.config.chain_id) {
                    return Err("invalid transaction signature".to_string());
                }
            }
            let transaction_hash = merkle_tree_root(transactions);
            if transaction_hash == header.tx_hash {
                return Err("invalid transaction hash".to_string());
            }
        }
        self.verify_header(&header, false)?;
        Ok(())
    }

    /// TODO
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, String> {
        let message = Message::from(digest);
        match sign(&self.key_pair.secret(), &message) {
            Ok(signature) => Ok(signature.to_vec()),
            Err(_) => Err("invalid sign".to_string()),
        }
    }

    /// TODO
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()> {
        let keccak_hash = hash(data);
        let signature = Signature::from_slice(sig);
        verify_address(&address, &signature, &Message::from(keccak_hash.as_ref())).map_err(|_| ())
    }

    fn last_proposal(&self) -> Result<Proposal, ()> {
        let ledger = self.ledger.read().unwrap();
        let block = ledger.get_last_block();
        Ok(Proposal::new(block.clone()))
    }

    fn has_proposal(&self, hash: &Hash, height: Height) -> bool {
        let ledger = self.ledger.read().unwrap();
        let block_hash = ledger.get_block_hash_by_height(height);
        block_hash.map_or(EMPTY_HASH, |v| v) == *hash
    }

    fn get_proposer(&self, height: Height) -> Address {
        let header = {
            let ledger = self.ledger.read().unwrap();
            ledger.get_header_by_height(height)
        };
        header.map_or(*EMPTY_ADDRESS, |header| header.proposer)
    }

    // TODO
    fn parent_validators(&self, proposal: &Proposal) -> &ValidatorSet {
        &self.validator_set
    }

    /// TODO
    fn has_bad_proposal(&self, hash: Hash) -> bool {
        false
    }
}

impl<T> ImplBackend<T>
where
    T: ValidatorSet,
{
    fn verify_header(&self, header: &Header, seal: bool) -> Result<(), String> {
        if header.height == 0 {
            return Err("heigt is invalid".to_string());
        }
        let parent_header = {
            let ledger = self.ledger.read().unwrap();
            ledger
                .get_header_by_height(header.height)
                .ok_or("Lack of ancestors".to_string())?
        };
        if parent_header.hash() != header.prev_hash {
            return Err("parent hash != heaer.prev hash".to_string());
        }
        if header.time < parent_header.time + self.config.block_period {
            return Err("Invalid timestamp".to_string());
        }

        let votes = header.votes.as_ref().ok_or("lack votes".to_string())?;

        // FIXME add more check
        Ok(())
    }

    fn verify_committed(&self, header: &Header) {
    }
}
