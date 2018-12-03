use std::fs::File;
use std::io::{self, prelude::*};
use std::str::FromStr;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use actix::prelude::*;
use toml::Value as Toml;
use libp2p::{PeerId, Multiaddr};
use parking_lot::RwLock;

use crate::{
    logger::init_log,
    p2p::{
        discover_service::DiscoverService,
        P2PEvent,
        spawn_sync_subscriber,
        server::Server,
    },
    config::Config,
    core::tx_pool::{BaseTxPool, TxPool},
};

pub fn start_node(config: &str, sender: Sender<()>) -> Result<(), String> {
    init_log();
    let result = init_config(config);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let config = result.unwrap();
    let tx_pool = Arc::new(RwLock::new(init_transaction_pool(&config)));

    let _: JoinHandle<Result<(), String>> = spawn(move || {
        let system = System::new("bft-rs");
        init_p2p_service(&config);
        init_tcp_server(&config);
        crate::util::TimerRuntime::new(Duration::from_secs(50));
        system.run();
        sender.send(()).unwrap();
        Ok(())
    });

    Ok(())
}


fn init_p2p_service(config: &Config) {
    let p2p_subscriber = spawn_sync_subscriber();
    let peer_id = PeerId::from_str(&config.peer_id).unwrap();
    let mul_addr = Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap();
    let discover_service = DiscoverService::spawn_discover_service(p2p_subscriber, peer_id, mul_addr, config.ttl);
    info!("Init p2p service successfully");
}

fn init_tcp_server(config: &Config) {
    let peer_id = PeerId::from_str(&config.peer_id).unwrap();
    let mul_addr = Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap();
    Server::create(|ctx| {
        let pid = ctx.address();
        Server::new(Some(pid), peer_id, mul_addr, None)
    });
    info!("Init tcp server successfully");
}

fn init_config(config: &str) -> Result<Config, String> {
    info!("Init config: {}", config);
    let mut input = String::new();
    File::open(config).and_then(|mut f| {
        f.read_to_string(&mut input)
    }).map(|_| {
        toml::from_str::<Config>(&input).unwrap()
    }).map_err(|err| err.to_string())
}

fn init_transaction_pool(_: &Config) -> Box<TxPool> {
    info!("Init transaction pool successfully");
    Box::new(BaseTxPool::new())
}