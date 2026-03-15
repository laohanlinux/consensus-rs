use std::sync::Arc;

use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::KeyPair;
use crossbeam::channel::Receiver;

use super::{
    error::EngineResult,
    types::Proposal,
    pbft::core::core::Core,
    pbft::core::runner::CoreHandle,
    backend::new_impl_backend,
};

use crate::{
    subscriber::events::{BroadcastEventBus},
    types::block::{Block, Header},
    core::chain::Chain,
};

pub trait Engine {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn author(&self, header: &Header) -> Result<Address, String>;
    fn verify_header(&self, header: &Header, seal: bool) -> EngineResult;
    fn verify_seal(&self, header: &Header) -> EngineResult;
    fn new_chain_header(&mut self, proposal: &Proposal) -> EngineResult;
    fn prepare(&mut self, header: &mut Header) -> Result<(), String>;
    fn finalize(&mut self, header: &Header) -> Result<(), String>;
    fn seal(&mut self, new_block: &mut Block, abort: Receiver<()>) -> EngineResult;
}

pub type SafeEngine = Box<dyn Engine + Send + Sync>;

pub fn create_bft_engine(key_pair: KeyPair, chain: Arc<Chain>, broadcast_bus: BroadcastEventBus) -> (CoreHandle, SafeEngine) {
    info!("Create bft consensus engine");
    let mut backend = new_impl_backend(key_pair.clone(), chain.clone(), broadcast_bus);

    let (core_tx, core_rx) = crossbeam::channel::unbounded();
    let core_handle = CoreHandle::new(core_tx);
    let core_handle_for_run = core_handle.clone();

    let chain_clone = chain.clone();
    let backend_clone = backend.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(Core::run(
            chain_clone,
            backend_clone,
            key_pair,
            core_rx,
            core_handle_for_run,
        ));
    });

    backend.set_core_handle(core_handle.clone());
    let engine_backend: SafeEngine = Box::new(backend);
    (core_handle, engine_backend)
}
