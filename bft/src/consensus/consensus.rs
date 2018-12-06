use cryptocurrency_kit::ethkey::Address;
use crossbeam::Receiver;

use super::error::{EngineError, EngineResult};
use super::types::Proposal;
use crate::types::block::{Block, Header};

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

//pub fn create_consensus_engine() -> Box<Engine> {}
