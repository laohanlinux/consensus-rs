use cryptocurrency_kit::ethkey::Address;

use crate::types::block::Header;

struct BftConfig {
    request_time: u64,
    block_period: u64,
}

pub trait Engine {
    fn start(&mut self) -> Result<(), ()>;
    fn stop(&mut self) -> Result<(), ()>;
    fn author(&self, header: &Header) -> Result<Address, ()>;
    fn verify_header(&self, header: &Header, seal: bool) -> Result<(), ()>;
    fn verify_seal(&self, header: &Header) -> Result<(), ()>;
    fn prepare(&mut self, header: &Header) -> Result<(), ()>;
    fn finalize(&mut self, header: &Header) -> Result<(), ()>;
}