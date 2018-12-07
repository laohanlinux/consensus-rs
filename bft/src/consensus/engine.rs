use cryptocurrency_kit::ethkey::Address;
use crossbeam::crossbeam_channel::Sender;

use crate::types::block::{Header, Block};

//pub trait Engine {
//    fn author(&self, header: &Header)-> Result<Address, String>;
//    fn verify_header(&self, header: &Header, seal: bool) -> Result<(), String>;
//    fn verify_seal(&self, header: &Header) -> Result<(), String>;
//
//    // preare initializes the consensus fields of a block header according to the rules of a
//    // particular engine. The changes are executed inline.
//    fn prepare(&mut self, header: &Header) -> Result<(), String>;
//
//    fn finalize(&mut self, header: &Header) -> Result<Block, String>;
//
//    // seal generate a new block for the given input block with the local miner's
//    // seal place on top
//    fn seal(&self, block: &Block, stop: Sender<Block>) -> Result<Block, String>;
//}


// Handler should be implemented is the consensus needs to handle and send peer's message
pub trait Handler{
    // NewChainHeader handles a new head block comes
    fn new_chain_head(&self) -> Result<(), String>;

    fn handle_message(&self, address: Address, data: &[u8]) -> Result<bool, String>;
}