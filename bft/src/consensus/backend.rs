use std::sync::Arc;
use std::time::Duration;

use actix::prelude::*;
use futures::Future;
use parking_lot::RwLock;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::Local;
use chrono_humanize::HumanTime;
use crossbeam::crossbeam_channel::{self, Sender, Receiver};
use cryptocurrency_kit::common::to_keccak;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash, EMPTY_HASH, HASH_SIZE};
use cryptocurrency_kit::ethkey::keccak::Keccak256;
use cryptocurrency_kit::ethkey::{
    sign, verify_address, Address, KeyPair, Message, Public, Secret, Signature,
};
use lru_time_cache::LruCache;

use super::{
    core::core::Core,
    events::{NewHeaderEvent, FinalCommittedEvent, OpCMD},
    config::Config,
    consensus::Engine,
    error::{EngineError, EngineResult},
    types::Proposal,
    validator::{ImplValidatorSet, ValidatorSet, fn_selector},
};
use crate::{
    core::ledger::Ledger,
    core::chain::Chain,
    common::merkle_tree_root,
    types::block::{Block, Header},
    types::{Height, Validator, EMPTY_ADDRESS},
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
    fn commit(&mut self, proposal: &mut Proposal, seals: Vec<Signature>) -> Result<(), String>;
    /// verifies the proposal. If a err_future_block error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), EngineError>);
    fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, String>;
    fn check_signature(&self, data: &[u8], address: Address, sig: &[u8]) -> Result<bool, ()>;

    fn last_proposal(&self) -> Result<Proposal, ()>;
    fn has_proposal(&self, hash: &Hash, height: Height) -> bool;
    fn get_proposer(&self, height: Height) -> Address;
    fn parent_validators(&self, proposal: &Proposal) -> &Self::ValidatorsType;
    fn has_bad_proposal(&self, hash: Hash) -> bool;

    fn get_header_by_height(&self, height: Height) -> Option<Header>;
}

pub fn new_impl_backend(keypair: KeyPair, chain: Arc<Chain>) -> ImplBackend {
    let request_time = chain.config.request_time.as_millis();
    let block_period = chain.config.block_period.as_millis();
    let config = Config {
        request_time: request_time as u64,
        block_period: block_period as u64,
        chain_id: 0,
    };

    let addresses: Vec<Address> = chain.get_validators(chain.get_last_height())
        .iter()
        .map(|validator| *validator.address())
        .collect();
    let validator_set = ImplValidatorSet::new(&addresses, Box::new(fn_selector));
    let inbound_cache = LruCache::with_capacity(1 << 10);
    let outbound_cache = LruCache::with_capacity(1 << 10);
    let proposed_block_hash = EMPTY_HASH;
    let (tx, rx) = crossbeam_channel::bounded(1);

    ImplBackend {
        core_pid: None,
        started: false,
        validaor: Validator::new(keypair.address()),
        validator_set: validator_set,
        key_pair: keypair,
        inbound_cache: inbound_cache,
        outbound_cache: outbound_cache,
        proposed_block_hash: proposed_block_hash,
        commit_tx: tx,
        commit_rx: rx,
        ledger: chain.get_ledger().clone(),
        config: config,
    }
}

#[derive(Clone)]
pub struct ImplBackend {
    core_pid: Option<Addr<Core>>,
    validaor: Validator,
    validator_set: ImplValidatorSet,
    key_pair: KeyPair,
    inbound_cache: LruCache<Hash, String>,
    outbound_cache: LruCache<Hash, String>,
    proposed_block_hash: Hash,
    // proposal hash it from local node
    commit_tx: Sender<Block>,
    commit_rx: Receiver<Block>,
    ledger: Arc<RwLock<Ledger>>,
    started: bool,
    config: Config,
}

impl Backend for ImplBackend {
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
    fn commit(&mut self, proposal: &mut Proposal, seals: Vec<Signature>) -> Result<(), String> {
        // write seal into block
        proposal.set_seal(seals.clone());
        let block = proposal.block();
        if self.proposed_block_hash == block.hash() {
            let block = block.clone();
            self.commit_tx.send(block).unwrap();
        }
        let mut block = proposal.block().clone();
        let mut votes = block.mut_votes();
        votes.unwrap().add_votes(&seals);
        let mut ledger = self.ledger.write();
        ledger.add_block(&block);
        info!(
            "committed a new block, hash:{}, height:{}, proposer:{}",
            block.hash().short(),
            block.height(),
            block.coinbase()
        );
        // TODO add block broadcast
        Err("".to_owned())
    }

    /// TODO
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), EngineError>) {
        let block = &proposal.0;
        let header = block.header();
        let blh = header.hash();
        if self.has_bad_proposal(blh) {
            return (Duration::from_nanos(0), Err(EngineError::InvalidProposal));
        }

        // check transaction
        {
            let transactions = block.transactions().to_vec();
            for transaction in &transactions {
                if !transaction.verify_sign(self.config.chain_id) {
                    return (Duration::from_nanos(0), Err(EngineError::InvalidSignature));
                }
            }
            let transaction_hash = merkle_tree_root(transactions);
            if transaction_hash == header.tx_hash {
                return (
                    Duration::from_nanos(0),
                    Err(EngineError::InvalidTransactionHash),
                );
            }
        }
        let result = self.verify_header(&header, false);
        if let Err(ref err) = result {
            match err {
                EngineError::FutureBlock => {
                    let now = Local::now().timestamp() as u64;
                    if now <= block.header().time {
                        return (Duration::from_nanos(now - block.header().time), result);
                    } else {
                        return (Duration::from_nanos(0), result);
                    }
                }
                _ => return (Duration::from_nanos(0), result),
            }
        }
        (Duration::from_nanos(0), Ok(()))
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
        let ledger = self.ledger.read();
        let block = ledger.get_last_block();
        Ok(Proposal::new(block.clone()))
    }

    fn has_proposal(&self, hash: &Hash, height: Height) -> bool {
        let ledger = self.ledger.read();
        let block_hash = ledger.get_block_hash_by_height(height);
        block_hash.map_or(EMPTY_HASH, |v| v) == *hash
    }

    fn get_proposer(&self, height: Height) -> Address {
        let header = {
            let ledger = self.ledger.read();
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

    fn get_header_by_height(&self, height: Height) -> Option<Header> {
        let ledger = self.ledger.read();
        ledger.get_header_by_height(height)
    }
}

impl Engine for ImplBackend {
    fn start(&mut self) -> Result<(), String> {
        if self.started {
            panic!("Engine start only once");
        }
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        let request = self.core_pid.as_ref().unwrap().send(OpCMD::stop);
        Arbiter::spawn(request
            .and_then(|_| futures::future::ok(()))
            .map_err(|err| panic!(err)));
        self.core_pid = None;
        self.started = false;
        Ok(())
    }

    // return the proposer
    fn author(&self, header: &Header) -> Result<Address, String> {
        Ok(header.proposer.clone())
    }

    fn verify_header(&self, header: &Header, seal: bool) -> Result<(), EngineError> {
        if header.height == 0 {
            return Err(EngineError::InvalidHeight);
        }
        let parent_header = {
            let ledger = self.ledger.read();
            ledger
                .get_header_by_height(header.height)
                .ok_or(EngineError::UnknownAncestor)?
        };
        if parent_header.hash() != header.prev_hash {
            return Err(EngineError::Unknown(
                "parent hash != heaer.prev hash".to_string(),
            ));
        }
        if header.time < parent_header.time + self.config.block_period {
            return Err(EngineError::InvalidTimestamp);
        }

        // check votes
        {
            let votes = header.votes.as_ref().ok_or(EngineError::LackVotes(
                self.validator_set.two_thirds_majority() + 1,
                header.votes.as_ref().unwrap().len(),
            ))?;
            if votes.verify_signs(CryptoHash::hash(header), |validator| {
                self.validator_set.get_by_address(validator).is_some()
            }) == false
                {
                    return Err(EngineError::InvalidSignature);
                }
            let maj32 = self.validator_set.two_thirds_majority();
            if maj32 + 1 > votes.len() {
                return Err(EngineError::LackVotes(maj32 + 1, votes.len()));
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
        self.validator_set
            .get_by_address(proposer)
            .ok_or("proposer is not validators".to_string())
            .map(|_| ())
    }

    fn new_chain_header(&mut self, proposal: &Proposal) -> EngineResult {
        trace!("Backend handle new chain header, hash: {:?}, height: {:?}", proposal.block().hash(), proposal.block().height());
        if !self.started {
            return Err(EngineError::EngineNotStarted);
        }
        // send a new round event
        let core = self.core_pid.as_ref().unwrap();
        let (tx, rx) = crossbeam_channel::bounded(1);
        let request = core.send(NewHeaderEvent { proposal: proposal.clone() });
        Arbiter::spawn(request.and_then(move |result| {
            tx.send(result);
            futures::future::ok(())
        }).map_err(|err| panic!(err)));
        rx.recv().unwrap();
        Ok(())
    }

    fn prepare(&mut self, header: &mut Header) -> Result<(), String> {
        let parent_header = {
            let ledger = self.ledger.read();
            ledger
                .get_header_by_height(header.height - 1)
                .ok_or("not found parent block for the header".to_string())?
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
    fn finalize(&mut self, header: &Header) -> Result<(), String> {
        self.proposed_block_hash = EMPTY_HASH;
        // send a new round event
        let core = self.core_pid.as_ref().unwrap();
        let (tx, rx) = crossbeam_channel::bounded(1);
        let request = core.send(FinalCommittedEvent {});
        Arbiter::spawn(request.and_then(move |result| {
            tx.send(result);
            futures::future::ok(())
        }).map_err(|err| panic!(err)));
        rx.recv().unwrap();
        Ok(())
    }

    fn seal(&mut self, new_block: &mut Block, abort: Receiver<()>) -> Result<Block, EngineError> {
        if !self.started {
            return Err(EngineError::EngineNotStarted);
        }

        let header = new_block.mut_header();

        // TODO update new validator
        // TODO add sign
        let delay = {
            let now = chrono::Local::now().timestamp() as u64;
            if now < header.time {
                header.time - now
            } else {
                0
            }
        };

        info!("Wait for the timestamp of header for ajustting the block period, delay: {}", delay);
        ::std::thread::sleep(Duration::from_secs(delay));

        // add clear function
        self.prepare(header).unwrap();
        // ready to new consensus
        self.new_chain_header(&Proposal(new_block.clone())).unwrap();
        let commit_tx = self.commit_rx.clone();

        let mut receover = || {
            self.finalize(new_block.header()).unwrap();
        };

        loop {
            select! {
                recv(abort) -> _ =>  {
                    receover();
                    return Err(EngineError::Interrupt);
                },
                recv(commit_tx) -> result => {
                    let block = result.unwrap();
                    let got = block.height() <= new_block.height();
                    assert!(got);

                    if block.hash() == new_block.hash() {
                        receover();
                        return Ok(block);
                    }
                },
            }
        }
        unimplemented!();
    }
}
