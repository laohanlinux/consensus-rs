use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::spawn;

use cryptocurrency_kit::ethkey::{KeyPair, Secret};
use kvdb_rocksdb::Database;
use lru_time_cache::LruCache;
use parking_lot::RwLock;

use crate::{
    common,
    config::Config,
    consensus::consensus::{create_bft_engine, SafeEngine},
    consensus::pbft::core::core::handle_msg_middle,
    core::chain::Chain,
    core::ledger::{LastMeta, Ledger},
    core::tx_pool::{BaseTxPool, SafeTxPool},
    error::ChainResult,
    logger::init_log,
    minner::start_minner,
    p2p::{
        discover_service::DiscoverService,
        server::{author_handshake, TcpServer},
        spawn_sync_subscriber,
    },
    pprof::spawn_signal_handler,
    store::schema::Schema,
    subscriber::events::BroadcastEventBus,
    types::Validator,
    api::start_api,
};

pub fn start_node(config: &str, _sender: Sender<()>) -> Result<(), String> {
    print_art();
    init_log();
    let result = init_config(config);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let config = result.unwrap();
    let secret = Secret::from_str(&config.secret).expect("Secret is uncorrect");
    let key_pair = KeyPair::from_secret(secret).unwrap();
    let ledger = init_store(&config)?;
    let ledger: Arc<RwLock<Ledger>> = Arc::new(RwLock::new(ledger));

    let mut chain = Chain::new(config.clone(), ledger);

    // init genesis
    init_genesis(&mut chain).map_err(|err| format!("{}", err))?;
    info!("Genesis hash: {:?}", chain.get_genesis().hash());

    // init transaction pool
    let tx_pool = Arc::new(RwLock::new(init_transaction_pool(&config)));

    let chain = Arc::new(chain);

    init_api(&config, chain.clone());

    let broadcast_bus = BroadcastEventBus::new(1024);

    let (core_handle, mut engine) = start_consensus_engine(
        &config,
        key_pair.clone(),
        chain.clone(),
        broadcast_bus.clone(),
    );
    engine.start()?;

    let config_clone = config.clone();
    {
        let p2p_event_bus = spawn_sync_subscriber();
        let p2p_event_bus_for_discover = p2p_event_bus.clone();
        std::thread::spawn(move || {
            DiscoverService::run_discover_service(
                p2p_event_bus_for_discover,
                libp2p::PeerId::from_str(&config_clone.peer_id).unwrap(),
                libp2p::Multiaddr::from_str(&format!("/ip4/{}/tcp/0", config_clone.ip)).unwrap(),
                config_clone.ttl,
            );
        });
        let genesis = chain.get_genesis().hash();
        let mut p2p_rx = p2p_event_bus.subscribe();
        let (server, _handle) = TcpServer::new(
            libp2p::PeerId::from_str(&config.peer_id).unwrap(),
            libp2p::Multiaddr::from_str(&format!("/ip4/{}/tcp/{}", config.ip, config.port)).unwrap(),
            None,
            genesis,
            Box::new(author_handshake(genesis)),
            Box::new(handle_msg_middle(core_handle.clone(), chain.clone())),
        );
        let local_peer_id = libp2p::PeerId::from_str(&config.peer_id).unwrap();
        for bp in &config.bootstrap_peers {
            if let (Ok(peer_id), Ok(multiaddr)) = (
                libp2p::PeerId::from_str(&bp.peer_id),
                libp2p::Multiaddr::from_str(&bp.multiaddr),
            ) {
                if peer_id != local_peer_id {
                    p2p_event_bus.send(crate::subscriber::P2PEvent::AddPeer(peer_id, vec![multiaddr]));
                }
            }
        }
        let chain_bus = chain.chain_event_bus();
        let mut chain_rx = chain_bus.subscribe();
        let server_for_chain = server.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                while let Ok(event) = chain_rx.recv().await {
                    server_for_chain.handle_chain_event(event);
                }
            });
        });
        let mut broadcast_rx = broadcast_bus.subscribe();
        let server_for_broadcast = server.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                while let Ok(event) = broadcast_rx.recv().await {
                    server_for_broadcast.handle_broadcast_event(event);
                }
            });
        });
        let server_for_p2p = server.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                while let Ok(event) = p2p_rx.recv().await {
                    match event {
                        crate::subscriber::P2PEvent::AddPeer(peer_id, addrs) => server_for_p2p.add_peer(peer_id, addrs),
                        crate::subscriber::P2PEvent::DropPeer(_, _) => {}
                    }
                }
            });
        });
    }

    // spawn thread for mining
    std::thread::spawn(move || {
        start_minner(&config, key_pair.clone(), chain.clone(), tx_pool.clone(), engine);
    });

    init_signal_handle();
    Ok(())
}

fn init_config(config: &str) -> Result<Config, String> {
    info!("Init config: {}", config);
    let mut input = String::new();
    File::open(config)
        .and_then(|mut f| f.read_to_string(&mut input))
        .map(|_| toml::from_str::<Config>(&input).unwrap())
        .map_err(|err| err.to_string())
}

fn init_transaction_pool(_: &Config) -> SafeTxPool {
    info!("Init transaction pool successfully");
    Box::new(BaseTxPool::new()) as SafeTxPool
}

fn init_store(config: &Config) -> Result<Ledger, String> {
    info!("Init store: {}", config.store);
    let genesis_config = config.genesis.as_ref().unwrap();

    let mut validators: Vec<Validator> = vec![];
    for validator in &genesis_config.validator {
        validators.push(Validator::new(common::string_to_address(validator)?));
    }

    let database = Database::open(&crate::store::schema::database_config(), &config.store)
        .map_err(|err| err.to_string())?;
    let schema = Schema::new(Arc::new(database));
    Ok(Ledger::new(
        LastMeta::new_zero(),
        LruCache::with_capacity(1 << 10),
        LruCache::with_capacity(1 << 10),
        validators,
        schema,
    ))
}

fn init_genesis(chain: &mut Chain) -> ChainResult {
    info!("Init genesis block");
    chain.store_genesis_block()
}

fn start_consensus_engine(
    _config: &Config,
    key_pair: KeyPair,
    chain: Arc<Chain>,
    broadcast_bus: BroadcastEventBus,
) -> (crate::consensus::pbft::core::runner::CoreHandle, SafeEngine) {
    info!("Init consensus engine");
    create_bft_engine(key_pair, chain, broadcast_bus)
}

fn init_api(config: &Config, chain: Arc<Chain>) {
    let config = config.clone();
    let chain = chain.clone();
    spawn(move || {
        info!("Start service api");
        start_api(chain, config.api_ip, config.api_port);
    });
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
