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
use lru_time_cache::LruCache;
use kvdb_rocksdb::Database;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::{KeyPair, Generator, Random};

use crate::{
    logger::init_log,
    p2p::{
        discover_service::DiscoverService,
        P2PEvent,
        spawn_sync_subscriber,
        server::{TcpServer, author_handshake, author_fn},
    },
    config::Config,
    core::tx_pool::{BaseTxPool, TxPool},
    subscriber::*,
    common,
    consensus::consensus::{Engine, create_consensus_engine},
    core::ledger::{Ledger, LastMeta},
    core::chain::Chain,
    error::ChainResult,
    store::schema::Schema,
    pprof::spawn_signal_handler,
    types::Validator,
    subscriber::events::{ChainEvent, SubscriberType, ChainEventSubscriber},
};

pub fn start_node(config: &str, sender: Sender<()>) -> Result<(), String> {
    print_art();
    init_log();
    let result = init_config(config);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let config = result.unwrap();
    println!("--> {:?}", config);
    let ledger = init_store(&config)?;
    let ledger: Arc<RwLock<Ledger>> = Arc::new(RwLock::new(ledger));
    // init subscriber
    println!("----------");
    let sub = ChainEventSubscriber::new(SubscriberType::Async).start();
    println!("---|||-------");
    let mut chain = Chain::new(config.clone(), sub.clone(), ledger);
    init_genesis(&mut chain).map_err(|err| format!("{}", err))?;
    let genesis = chain.get_genesis().clone();
    info!("Genesis hash: {:?}", chain.get_genesis().hash());
    let tx_pool = Arc::new(RwLock::new(init_transaction_pool(&config)));

    let chain = Arc::new(chain);
    start_consensus_engine(&config, Random.generate().unwrap(), chain.clone());

    let _: JoinHandle<Result<(), String>> = spawn(move || {
        let system = System::new("bft-rs");
        let p2p_event_notify = init_p2p_event_notify();
        let discover_pid = init_p2p_service(p2p_event_notify.clone(), &config);
        init_tcp_server(p2p_event_notify.clone(), genesis.hash(), &config);
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

fn init_tcp_server(p2p_subscriber: Addr<ProcessSignals>, genesis: Hash, config: &Config) {
    let peer_id = PeerId::from_str(&config.peer_id).unwrap();
    let mul_addr = Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap();
    let author = author_handshake(genesis.clone());
    let server = TcpServer::new(peer_id, mul_addr, None, genesis.clone(), Box::new(author));

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

fn init_store(config: &Config) -> Result<Ledger, String> {
    info!("Init store: {}", config.store);
    let genesis_config = config.genesis.as_ref().unwrap();

    let mut validators: Vec<Validator> = vec![];
    for validator in &genesis_config.validator {
        validators.push(Validator::new(common::string_to_address(validator)?));
    }

    let database = Database::open_default(&config.store).map_err(|err| err.to_string())?;
    let schema = Schema::new(Arc::new(database));
    Ok(Ledger::new(
        LastMeta::new_zero(),
        LruCache::with_capacity(1 << 10),
        LruCache::with_capacity(1 << 10),
        validators,
        schema))
}

fn init_genesis(chain: &mut Chain) -> ChainResult {
    info!("Init genesis block");
    chain.store_genesis_block()
}

fn start_consensus_engine(config: &Config, key_pair: KeyPair, chain: Arc<Chain>) -> Box<Engine> {
    info!("Init consensus engine");
    create_consensus_engine(key_pair, chain)
}

fn init_signal_handle() {
    spawn_signal_handler(*common::random_dir());
}

fn print_art() {
    let art = r#"
    A large collection of ASCII art drawings of bears and other related animal ASCII art pictures.

    lazy bears by Joan G. Stark

    _,-""`""-~`)
    (`~_,=========\
    |---,___.-.__,\
    |        o     \ ___  _,,,,_     _.--.
    \      `^`    /`_.-"~      `~-;`     \
       \_      _  .'                 `,     |
         |`-                           \'__/
        /                      ,_       \  `'-.
       /    .-""~~--.            `"-,   ;_    /
    |              \               \  | `""`
    \__.--'`"-.   /_               |'
                  `"`  `~~~---..,     |
                                 \ _.-'`-.
    "#;
    println!("{}", art);
}