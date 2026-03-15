use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use crossbeam::channel::{self, Receiver, RecvTimeoutError, Sender};
use cryptocurrency_kit::common::to_fixed_array_32;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::{CryptoHash, Hash, hash, EMPTY_HASH};
use cryptocurrency_kit::ethkey::{
    sign, verify_address, Address, KeyPair, Message, Signature,
};
use lru_time_cache::LruCache;

use super::{
    config::Config,
    consensus::Engine,
    pbft::core::runner::CoreHandle,
    error::{EngineError, EngineResult},
    types::Proposal,
    validator::{fn_selector, ImplValidatorSet, ValidatorSet},
};
use crate::{
    common::merkle_tree_root,
    core::chain::Chain,
    error::ChainError,
    protocol::GossipMessage,
    subscriber::events::{BroadcastEvent, BroadcastEventBus},
    types::block::{Block, Header},
    types::{Height, Validator, EMPTY_ADDRESS},
};
use ethereum_types::H256;

pub trait Backend {
    type ValidatorsType;
    /// address is the current validator's address
    fn address(&self) -> Address;
    /// validators returns a set of current validator
    fn validators(&self, height: Height) -> &Self::ValidatorsType;
    /// gossip sends a message to all validators (exclude self)
    fn gossip(&mut self, vals: &dyn ValidatorSet, msg: GossipMessage) -> EngineResult;
    /// commit a proposal with seals
    fn commit(&mut self, proposal: &mut Proposal, seals: Vec<Signature>) -> Result<(), String>;
    /// verifies the proposal. If a err_future_block error is returned,
    /// the time difference of the proposal and current time is also returned.
    fn verify(&self, proposal: &Proposal) -> (Duration, Result<(), EngineError>);
    fn sign(&self, digest: &[u8; 32]) -> Result<Vec<u8>, String>;
    fn check_signature(&self, data: &[u8; 32], address: Address, sig: &[u8]) -> Result<bool, ()>;

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
    broadcast_bus: BroadcastEventBus,
) -> ImplBackend {
    let request_time = chain.config.request_time.as_millis();
    let block_period = chain.config.block_period.as_secs();
    let config = Config {
        request_time: request_time as u64,
        block_period,
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
    let (tx, rx) = channel::bounded(1);

    ImplBackend {
        core_handle: None,
        broadcast_bus,
        started: false,
        validaor: Validator::new(keypair.address()),
        validator_set,
        key_pair: keypair,
        inbound_cache,
        outbound_cache,
        proposed_block_hash,
        commit_tx: tx,
        commit_rx: rx,
        chain,
        config,
    }
}

#[derive(Clone)]
pub struct ImplBackend {
    core_handle: Option<CoreHandle>,
    broadcast_bus: BroadcastEventBus,
    validaor: Validator,
    validator_set: ImplValidatorSet,
    key_pair: KeyPair,
    #[allow(dead_code)]
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
    fn gossip(&mut self, _vals: &dyn ValidatorSet, msg: GossipMessage) -> EngineResult {
        let msg_hash = msg.hash();
        if self.outbound_cache.get(&msg_hash).is_some() {
            debug!("The message has sent");
            return Ok(());
        }
        debug!("Broadcast message, {:?}", msg.trace());

        self.outbound_cache.insert(msg_hash, ());
        let core_handle = self.core_handle.as_ref().ok_or(EngineError::EngineNotStarted)?;
        core_handle.send_message(msg.clone().into_bytes());
        self.broadcast_bus.send(BroadcastEvent::Consensus(msg));
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
                _other => {
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
                    Err(EngineError::InvalidTransactionHash(header.tx_hash, transaction_hash)),
                );
            }
        }
        let result = self.verify_header(header, false);
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
    fn sign(&self, digest: &[u8; 32]) -> Result<Vec<u8>, String> {
        let message = Message::from(digest);
        match sign(self.key_pair.secret(), &message) {
            Ok(signature) => Ok(signature.to_vec()),
            Err(_) => Err("invalid sign".to_string()),
        }
    }

    /// TODO
    fn check_signature(&self, data: &[u8; 32], address: Address, sig: &[u8]) -> Result<bool, ()> {
        let keccak_hash = H256::from(to_fixed_array_32(hash(data).as_ref()));
        let signature = Signature::from_slice(sig);
        verify_address(&address, &signature, &Message::from(keccak_hash)).map_err(|_| ())
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
        if let Some(ref h) = self.core_handle {
            h.send_stop();
        }
        self.core_handle = None;
        self.started = false;
        Ok(())
    }

    // return the proposer
    fn author(&self, header: &Header) -> Result<Address, String> {
        Ok(header.proposer)
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
            if !votes.verify_signs(CryptoHash::hash(header), |validator| {
                self.validator_set.get_by_address(validator).is_some()
            })
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
        let core = self.core_handle.as_ref().unwrap().clone();
        let proposal = proposal.clone();
        core.send_new_header(proposal);
        Ok(())
    }

    fn prepare(&mut self, header: &mut Header) -> Result<(), String> {
        debug!(
            "Excute prepare work, header, hash:{:?}, height:{:?}",
            header.block_hash(),
            header.height
        );
        let _parent_header = {
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
        let core = self.core_handle.as_ref().unwrap().clone();
        core.send_final_committed();
        Ok(())
    }

    fn seal(&mut self, new_block: &mut Block, abort: Receiver<()>) -> EngineResult {
        if !self.started {
            return Err(EngineError::EngineNotStarted);
        }

        let header = new_block.mut_header();

        let delay_ms = {
            let now_ms = chrono::Local::now().timestamp_millis() as u64;
            let target_ms = header.time * 1000;
            target_ms.saturating_sub(now_ms)
        };

        info!(
            "⛏️⛏️⛏👷️ Minnig next block, hash:{:?}, height:{:?}, delay: {}ms",
            header.block_hash().short(), header.height, delay_ms);
        ::std::thread::sleep(Duration::from_millis(delay_ms));

        self.prepare(header).unwrap();
        self.new_chain_header(&Proposal(new_block.clone())).unwrap();
        let commit_rx = self.commit_rx.clone();
        let new_hash = new_block.hash();
        let new_height = new_block.height();

        let mut wait_count: u64 = 0;
        loop {
            if abort.try_recv().is_ok() {
                trace!("seal abort, height={}", new_height);
                return Err(EngineError::Interrupt);
            }
            match commit_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(block) => {
                    trace!("seal got block height={} hash={:?}", block.height(), block.hash().short());
                    assert!(block.height() <= new_height);
                    if block.hash() == new_hash {
                        self.finalize(new_block.header()).unwrap();
                        break Ok(());
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    wait_count += 1;
                    if wait_count % 10 == 1 {
                        trace!("seal waiting for commit height={} (waited {}s)", new_height, wait_count);
                    }
                    continue;
                }
                Err(_) => {
                    trace!("seal commit_rx closed, height={}", new_height);
                    return Err(EngineError::Interrupt);
                }
            }
        }
    }
}

impl ImplBackend {
    pub fn set_core_handle(&mut self, handle: CoreHandle) {
        self.core_handle = Some(handle);
        trace!("Set core handle for backend");
    }
}
