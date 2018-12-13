use std::sync::Arc;

use actix::Addr;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::KeyPair;
use crossbeam::Receiver;

use super::{
    error::{EngineError, EngineResult},
    types::Proposal,
    core::core::Core,
    backend::{Backend, ImplBackend, new_impl_backend},
    validator::ImplValidatorSet,
};

use crate::{
    subscriber::events::{BroadcastEvent, BroadcastEventSubscriber},
    types::block::{Block, Header},
    core::chain::Chain,
    consensus::events::OpCMD,
};

struct BftConfig {
    request_time: u64,
    block_period: u64,
}

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

pub type SafeEngine = Box<Engine + Send + Sync>;

pub fn create_consensus_engine(key_pair: KeyPair, chain: Arc<Chain>, subscriber: Addr<BroadcastEventSubscriber>) -> (Addr<Core>, SafeEngine) {
    info!("Create bft consensus engine");
    let mut backend = new_impl_backend(key_pair.clone(), chain.clone(), subscriber);

    // use new thread to handle core
    let (tx, rx) = ::std::sync::mpsc::channel();
    let core_backend = backend.clone();
    ::std::thread::spawn(move || {
        let core = actix::System::run(move || {
            let core_pid = Core::new(chain, core_backend, key_pair);
            tx.send(core_pid).unwrap();
        });
        ::std::process::exit(core);
    });
    let core_pid = rx.recv().unwrap();
    backend.set_core_pid(core_pid.clone());
    let engine_backend: SafeEngine = Box::new(backend.clone()) as SafeEngine;
    (core_pid, engine_backend)
}
