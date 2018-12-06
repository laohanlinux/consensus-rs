use std::sync::Arc;

use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::KeyPair;
use crossbeam::Receiver;

use super::error::{EngineError, EngineResult};
use super::types::Proposal;
use crate::types::block::{Block, Header};
use crate::core::chain::Chain;
use super::core::core::Core;
use super::backend::{Backend, ImplBackend, new_impl_backend};
use super::validator::ImplValidatorSet;

struct BftConfig {
    request_time: u64,
    block_period: u64,
}

pub trait Engine {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn author(&self, header: &Header) -> Result<Address, String>;
    fn verify_header(&self, header: &Header, seal: bool) -> EngineResult;
    fn verify_seal(&self, header: &Header) -> Result<(), String>;
    fn new_chain_header(&mut self, proposal: &Proposal) -> EngineResult;
    fn prepare(&mut self, header: &mut Header) -> Result<(), String>;
    fn finalize(&mut self, header: &Header) -> Result<(), String>;
    fn seal(&mut self, new_block: &mut Block, abort: Receiver<()>) -> Result<Block, EngineError>;
}

pub fn create_consensus_engine(key_pair: KeyPair, chain: Arc<Chain>) -> Box<Engine> {
    info!("Create bft consensus engine");
    let backend = new_impl_backend(key_pair.clone(), chain.clone());
    let core_backend: Box<Backend<ValidatorsType=ImplValidatorSet>> = Box::new(backend.clone());
    let engine_backend: Box<Engine> = Box::new(backend.clone());
    let core_pid = Core::new(chain, core_backend, key_pair);
    engine_backend
}
