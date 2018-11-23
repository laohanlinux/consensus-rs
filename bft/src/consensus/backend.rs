use lru_time_cache::LruCache;

use chrono::{Local, Duration};
use chrono_humanize::HumanTime;
use crossbeam::crossbeam_channel::Sender;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use cryptocurrency_kit::crypto::{hash, HASH_SIZE, CryptoHash, Hash, EMPTY_HASH};
use cryptocurrency_kit::ethkey::keccak::Keccak256;
use cryptocurrency_kit::ethkey::{
    sign, verify_address, Address, KeyPair, Message, Public, Secret, Signature,
};
use cryptocurrency_kit::common::to_keccak;

use std::sync::{Arc, RwLock};

use super::{
    config::Config,
    types::Proposal,
    validator::{ImplValidatorSet, ValidatorSet},
    consensus::Engine,
};

use crate::{
    common::merkle_tree_root,
    store::ledger::Ledger,
    types::block::{Header, Block},
    types::transaction::Transaction,
    types::{Height, Validator, EMPTY_ADDRESS},
    protocol::MessageType,
};

pub trait Backend {
    type ValidatorsType;
    /// address is the current validator's address
    fn address(&self) -> Address;
    /// validators returns a set of current validator
    fn validators(&self, height: Height) -> &Self::ValidatorsType;
    ///TODO
    fn event_mux(&self);
    /// broadcast sends a message to all validators (include itself)
    fn broadcast(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// gossip sends a message to all validators (exclude self)
    fn gossip(&self, vals: &ValidatorSet, payload: &[u8]) -> Result<(), ()>;
    /// commit a proposal with seals
    fn commit(&mut self, proposal: &mut Proposal, seals: Vec<Signature>) -> Result<(), ()>;
    /// verifies the proposal. If a err_future_block error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> Result<(), String>;
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, String>;
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()>;

    fn last_proposal(&self) -> Result<Proposal, ()>;
    fn has_proposal(&self, hash: &Hash, height: Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators(&self, proposal: &Proposal) -> &Self::ValidatorsType;
    fn has_bad_proposal(&self, hash: Hash) -> bool;
}

struct ImplBackend {
    validaor: Validator,
    validator_set: ImplValidatorSet,
    key_pair: KeyPair,
    inbound_cache: LruCache<Hash, String>,
    outbound_cache: LruCache<Hash, String>,
    proposed_block_hash: Hash,
    // proposal hash it from local node
    commit_channel: Sender<Block>,
    ledger: Arc<RwLock<Ledger>>,
    config: Config,
}

impl Backend for ImplBackend
{
    type ValidatorsType = ImplValidatorSet;
    fn address(&self) -> Address {
        *self.validaor.address()
    }

    fn validators(&self, _: Height) -> &ImplValidatorSet {
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
    fn commit(&mut self, proposal: &mut Proposal, seals: Vec<Signature>) -> Result<(), ()> {
        // write seal into block
        proposal.set_seal(seals);
        let block = proposal.block();
        // 1. if the proposed and committed blocks are the same, send the proposed hash
        //  to commit channel, which is being watched inside the engine.Seal() function.
        // 2. otherwise, we try to insert the block.
        // 3. if success, the `chain head event` event will be broadcasted, try to build
        //  next block and the previous seal() will be stopped.
        // 4. otherwise, a error will be returned and a round change event will be fired.
        if self.proposed_block_hash == block.hash() {
            let block = block.clone();
            self.commit_channel.send(block).unwrap();
        }
        info!("committed a new block, hash:{}, height:{}, proposer:{}", block.hash().short(), block.height(), block.coinbase());

        // TODO add block broadcast
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
    fn parent_validators(&self, proposal: &Proposal) -> &Self::ValidatorsType {
        &self.validator_set
    }

    /// TODO
    fn has_bad_proposal(&self, hash: Hash) -> bool {
        false
    }
}

impl Engine for ImplBackend
{
    fn start(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }

    // return the proposer
    fn author(&self, header: &Header) -> Result<Address, String> {
        Ok(header.proposer.clone())
    }

    fn verify_header(&self, header: &Header, seal: bool) -> Result<(), String> {
        use std::io::{Read, Write, Cursor};
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

        // check votes
        {
            let votes = header.votes.as_ref().ok_or("lack votes".to_string())?;
            let op_code = MessageType::Commit;
            let digest = header.hash();
            let mut input = Cursor::new(vec![0_u8; 1 + HASH_SIZE]);
            input.write_u8(op_code as u8).unwrap();
            input.write(digest.as_ref()).unwrap();
            let buffer = input.into_inner();
            let digest: Hash = hash(buffer);
            if votes.verify_signs(digest, |validator| { self.validator_set.get_by_address(validator).is_some() }) == false {
                return Err("invalid votes".to_string());
            }
            let maj32 = self.validator_set.two_thirds_majority();
            if maj32 + 1 > votes.len() {
                return Err("lack votes".to_string());
            }
        }

        // FIXME add more check
        Ok(())
    }

    fn verify_seal(&self, header: &Header) -> Result<(), String> {
        if header.height == 0 {
            return Err("unkown block".to_string());
        }
        let proposer = header.proposer;
        self.validator_set.get_by_address(proposer).ok_or("proposer is not validators".to_string()).map(|_| ())
    }

    fn prepare(&mut self, header: &mut Header) -> Result<(), String> {
        let parent_header = {
            let ledger = self.ledger.read().unwrap();
            ledger.get_header_by_height(header.height - 1).ok_or("not found parent block for the header".to_string())?
        };
        // TODO maybe reset validator

        header.votes = None;
        header.time = parent_header.time + self.config.block_period;
        let now = Local::now().timestamp() as u64;
        if header.time < now {
            header.time = now;
        }

        self.proposed_block_hash = header.hash();
        Ok(())
    }

    // Finalize runs any post-transaction state modifications (e.g. block rewards)
    // and assembles the final block.
    //
    // Note, the block header and state database might be updated to reflect any
    // consensus rules that happen at finalization (e.g. block rewards).
    fn finalize(&mut self, header: &mut Header) -> Result<(), String> {
        self.proposed_block_hash = EMPTY_HASH;
        Ok(())
    }
}