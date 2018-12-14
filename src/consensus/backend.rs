use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use actix::{Addr, Arbiter};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::Local;
use chrono_humanize::HumanTime;
use crossbeam::crossbeam_channel::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use crossbeam::scope;
use cryptocurrency_kit::common::to_keccak;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::{CryptoHash, Hash, hash, EMPTY_HASH};
use cryptocurrency_kit::ethkey::{
    keccak::Keccak256,
    sign, verify_address, Address, KeyPair, Message, Public, Secret, Signature,
};
use futures::future::Err;
use futures::sync::oneshot;
use futures::Future;
use futures::*;
use lru_time_cache::LruCache;
use parking_lot::RwLock;
use tokio_threadpool::ThreadPool;

use super::{
    config::Config,
    consensus::Engine,
    pbft::core::core::Core,
    error::{EngineError, EngineResult},
    events::{MessageEvent, FinalCommittedEvent, NewHeaderEvent, OpCMD},
    types::Proposal,
    validator::{fn_selector, ImplValidatorSet, ValidatorSet},
};
use crate::{
    common::merkle_tree_root,
    core::chain::Chain,
    error::{ChainError, ChainResult},
    protocol::GossipMessage,
    subscriber::events::{BroadcastEvent, BroadcastEventSubscriber},
    types::block::{Block, Header},
    types::{Height, Validator, EMPTY_ADDRESS},
};

pub trait Backend {
    type ValidatorsType;
    /// address is the current validator's address
    fn address(&self) -> Address;
    /// validators returns a set of current validator
    fn validators(&self, height: Height) -> &Self::ValidatorsType;
    /// gossip sends a message to all validators (exclude self)
    fn gossip(&mut self, vals: &ValidatorSet, msg: GossipMessage) -> EngineResult;
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

pub fn new_impl_backend(
    keypair: KeyPair,
    chain: Arc<Chain>,
    subscriber: Addr<BroadcastEventSubscriber>,
) -> ImplBackend {
    let request_time = chain.config.request_time.as_millis();
    let block_period = chain.config.block_period.as_secs();
    let config = Config {
        request_time: request_time as u64,
        block_period: block_period as u64,
        chain_id: 0,
    };

    let addresses: Vec<Address> = chain
        .get_validators(chain.get_last_height())
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
        broadcast_subscriber: subscriber,
        started: false,
        validaor: Validator::new(keypair.address()),
        validator_set: validator_set,
        key_pair: keypair,
        inbound_cache: inbound_cache,
        outbound_cache: outbound_cache,
        proposed_block_hash: proposed_block_hash,
        commit_tx: tx,
        commit_rx: rx,
        chain: chain,
        config: config,
    }
}

#[derive(Clone)]
pub struct ImplBackend {
    core_pid: Option<Addr<Core>>,
    broadcast_subscriber: Addr<BroadcastEventSubscriber>,
    validaor: Validator,
    validator_set: ImplValidatorSet,
    key_pair: KeyPair,
    inbound_cache: LruCache<Hash, ()>,
    outbound_cache: LruCache<Hash, ()>,
    proposed_block_hash: Hash,
    // proposal hash it from local node
    commit_tx: Sender<Block>,
    commit_rx: Receiver<Block>,
    chain: Arc<Chain>,
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
    fn gossip(&mut self, vals: &ValidatorSet, msg: GossipMessage) -> EngineResult {
        let msg_hash = msg.hash();
        if self.outbound_cache.get(&msg_hash).is_some() {
            debug!("The message has sent");
            return Ok(());
        }
        debug!("Broadcast message, {:?}", msg.trace());

        self.outbound_cache.insert(msg_hash, ());
        let core_pid = self.core_pid.as_ref().ok_or(EngineError::EngineNotStarted)?;
        Arbiter::spawn(core_pid.send(MessageEvent { payload: msg.clone().into_bytes() }).then(|result| {
            if let Err(ref err) = result {
                error!("Failed to send message");
            }
            trace!("Success to send message");
            future::ok::<(), ()>(())
        }).map_err(|err| panic!(err)));
        self.broadcast_subscriber
            .do_send(BroadcastEvent::Consensus(msg));
        Ok(())
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
        let votes = block.mut_votes();
        votes.unwrap().add_votes(&seals);
        let result = self.chain.insert_block(&block);
        if let Err(err) = result {
            match err {
                ChainError::Exists(block_hash) => {
                    trace!("Block hash exists. hash: {:?}", block_hash);
                }
                other => {
                    error!(
                        "Failed to committed a new block, hash:{}, height:{}, proposer:{}",
                        block.hash().short(),
                        block.height(),
                        block.coinbase()
                    );
                }
            }
            return Ok(());
        }

        debug!(
            "Committed a new block, hash:{}, height:{}, proposer:{}",
            block.hash().short(),
            block.height(),
            block.coinbase()
        );
        // TODO add block broadcast
        Ok(())
    }

    /// TODO
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), EngineError>) {
        let block = &proposal.0;
        let header = block.header();
        let blh = header.block_hash();
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
            if transaction_hash != header.tx_hash {
                return (
                    Duration::from_nanos(0),
                    Err(EngineError::InvalidTransactionHash(header.tx_hash.clone(), transaction_hash)),
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
        let block = self.chain.get_last_block();
        Ok(Proposal::new(block))
    }

    fn has_proposal(&self, hash: &Hash, height: Height) -> bool {
        if let Some(block_hash) = self.chain.get_block_hash_by_height(height) {
            return block_hash == *hash;
        }
        false
    }

    fn get_proposer(&self, height: Height) -> Address {
        let header = self.chain.get_header_by_height(height);
        header.map_or(*EMPTY_ADDRESS, |header| header.proposer)
    }

    // TODO
    fn parent_validators(&self, _proposal: &Proposal) -> &Self::ValidatorsType {
        &self.validator_set
    }

    /// TODO
    fn has_bad_proposal(&self, _hash: Hash) -> bool {
        false
    }

    fn get_header_by_height(&self, height: Height) -> Option<Header> {
        self.chain.get_header_by_height(height)
    }
}

impl Engine for ImplBackend {
    fn start(&mut self) -> Result<(), String> {
        if self.started {
            panic!("Engine start only once");
        }
        self.started = true;
        info!("Engine start successfully");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        let request = self.core_pid.as_ref().unwrap().send(OpCMD::stop);
        Arbiter::spawn(
            request
                .and_then(|_| futures::future::ok(()))
                .map_err(|err| panic!(err)),
        );
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
            self.chain
                .get_header_by_height(header.height - 1)
                .ok_or(EngineError::UnknownAncestor(header.height, header.height - 1))?
        };
        if parent_header.block_hash() != header.prev_hash {
            return Err(EngineError::Unknown(
                format!("parent hash({:?}) != heaer.prev hash({:?})", parent_header.block_hash(), header.prev_hash),
            ));
        }
        if header.time < parent_header.time + self.config.block_period {
            return Err(EngineError::InvalidTimestamp);
        }
        if seal {
            self.verify_seal(header)?;
        }
        // FIXME add more check
        Ok(())
    }

    fn verify_seal(&self, header: &Header) -> EngineResult {
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

        let proposer = header.proposer;
        self.validator_set
            .get_by_address(proposer)
            .ok_or(EngineError::Unknown("proposer is not validators".to_string()))
            .map(|_| ())
    }

    fn new_chain_header(&mut self, proposal: &Proposal) -> EngineResult {
        debug!(
            "Backend handle new chain header, hash: {:?}, height: {:?}",
            proposal.block().hash(),
            proposal.block().height()
        );
        if !self.started {
            return Err(EngineError::EngineNotStarted);
        }
        // send a new round event
        let core = self.core_pid.as_ref().unwrap().clone();
        let proposal = proposal.clone();
        core.do_send(NewHeaderEvent {
            proposal: proposal.clone(),
        });
        Ok(())
    }

    fn prepare(&mut self, header: &mut Header) -> Result<(), String> {
        debug!(
            "Excute prepare work, header, hash:{:?}, height:{:?}",
            header.block_hash(),
            header.height
        );
        let parent_header = {
            self.chain
                .get_header_by_height(header.height - 1)
                .ok_or("not found parent block for the header".to_string())?
        };
        // TODO maybe reset validator

        header.votes = None;
        self.proposed_block_hash = header.block_hash();
        Ok(())
    }

    // Finalize runs any post-transaction state modifications (e.g. block rewards)
    // and assembles the final block.
    //
    // Note, the block header and state database might be updated to reflect any
    // consensus rules that happen at finalization (e.g. block rewards).
    fn finalize(&mut self, _header: &Header) -> Result<(), String> {
        self.proposed_block_hash = EMPTY_HASH;
        // send a new round event
        let core = self.core_pid.as_ref().unwrap();
        let (tx, rx) = crossbeam_channel::bounded(1);
        let request = core.send(FinalCommittedEvent {});
        Arbiter::spawn(
            request
                .and_then(move |result| {
                    tx.send(result);
                    futures::future::ok(())
                })
                .map_err(|err| panic!(err)),
        );
        rx.recv().unwrap();
        Ok(())
    }

    fn seal(&mut self, new_block: &mut Block, abort: Receiver<()>) -> EngineResult {
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

        info!(
            "⛏️⛏️⛏👷️ Minnig next block, hash:{:?}, height:{:?}, delay: {}s",
            header.block_hash().short(), header.height, delay);
        ::std::thread::sleep(Duration::from_secs(delay));

        // add clear function
        self.prepare(header).unwrap();
        // ready to new consensus
        self.new_chain_header(&Proposal(new_block.clone())).unwrap();
        let commit_tx = self.commit_rx.clone();

        let mut receover = || {
            self.finalize(new_block.header()).unwrap();
        };

        let tx = worker.sender().clone();
        let new_hash = new_block.hash().clone();
        let new_height = new_block.height();
        let res = oneshot::spawn(
            future::lazy(move || {
                while true {
                    if let Err(err) = abort.try_recv() {
                        match err {
                            TryRecvError::Disconnected => {
                                return futures::future::err(EngineError::Interrupt);
                            }
                            _ => {}
                        }
                    } else {
                        return futures::future::err(EngineError::Interrupt);
                    }

                    match commit_tx.recv_timeout(Duration::from_secs(1)) {
                        Ok(block) => {
                            let got = block.height() <= new_height;
                            assert!(got);

                            if block.hash() == new_hash {
                                return futures::future::ok(block);
                            }
                        }
                        Err(err) => match err {
                            RecvTimeoutError => {
                                continue;
                            }
                            other => {
                                panic!(other);
                            }
                        },
                    }
                }

                println!("Running on the pool");
                futures::future::err(EngineError::Interrupt)
            }),
            &tx,
        );

        let chain = self.chain.clone();
        Arbiter::spawn(
            res.then(move |res| {
                match res {
                    Ok(block) => {
//                        chain.insert_block(&block);
                    }
                    Err(err) => {
                        match err {
                            EngineError::Interrupt | EngineError::FutureBlock => {}
                            other => {
                                error!("Consensus fail, err:{:?}", other);
                            }
                        }
                    }
                }
                futures::future::ok::<(), String>(())
            })
                .map_err(|err| panic!(err)),
        );
        Ok(())
    }
}

impl ImplBackend {
    pub fn set_core_pid(&mut self, core_pid: Addr<Core>) {
        self.core_pid = Some(core_pid);
        trace!("Set core pid for backend");
    }
}

lazy_static! {
pub static ref worker: ThreadPool = ThreadPool::new();
}
