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
use futures::Future;

use crate::{
    logger::init_log,
    p2p::{
        discover_service::DiscoverService,
        P2PEvent,
        spawn_sync_subscriber,
        server::TcpServer,
    },
    config::Config,
    core::tx_pool::{BaseTxPool, TxPool},
    subscriber::*,
    common::random_dir,
    pprof::spawn_signal_handler,
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
        let p2p_event_notify = init_p2p_event_notify();
        let discover_pid = init_p2p_service(p2p_event_notify.clone(), &config);
        init_tcp_server(p2p_event_notify.clone(), &config);
        crate::util::TimerRuntime::new(Duration::from_secs(150));
        system.run();
        sender.send(()).unwrap();
        Ok(())
    });
    init_signal_handle();
    Ok(())
}


fn init_p2p_event_notify() -> Addr<ProcessSignals> {
    info!("Init p2p event nofity");
    spawn_sync_subscriber()
}

fn init_p2p_service(p2p_subscriber: Addr<ProcessSignals>, config: &Config) -> Addr<DiscoverService> {
    let peer_id = PeerId::from_str(&config.peer_id).unwrap();
    let mul_addr = Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap();
    let discover_service = DiscoverService::spawn_discover_service(p2p_subscriber, peer_id, mul_addr, config.ttl);
    info!("Init p2p service successfully");
    discover_service
}

fn init_tcp_server(p2p_subscriber: Addr<ProcessSignals>, config: &Config) {
    let peer_id = PeerId::from_str(&config.peer_id).unwrap();
    let mul_addr = Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap();
    let server = TcpServer::new(peer_id, mul_addr, None);


    // subscriber p2p event, sync operation
    {
        let recipient = server.recipient();
        // register
        let message = SubscribeMessage::SubScribe(recipient);
        let request_fut = p2p_subscriber.send(message);
        Arbiter::spawn(request_fut.and_then(|result| {
            info!("Subsribe p2p discover event successfully");
            futures::future::ok(())
        }).map_err(|err| unimplemented!("{}", err)));
    }
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

fn init_signal_handle() {
    spawn_signal_handler(*random_dir());
}