use cryptocurrency_kit::ethkey::Address;

use crate::types::block::Header;
use super::error::EngineResult;

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
    fn prepare(&mut self, header: &mut Header) -> Result<(), String>;
    fn finalize(&mut self, header: &mut Header) -> Result<(), String>;
}