use std::collections::HashMap;
use std::net;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use cryptocurrency_kit::crypto::{CryptoHash, Hash};
use cryptocurrency_kit::storage::values::StorageValue;
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use lru_time_cache::LruCache;
use parking_lot::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, sleep};

use super::protocol::{BoundType, RawMessage, Header as RawHeader, P2PMsgCode, Handshake};
use super::session::{Session, SessionTx};
use crate::{
    common::multiaddr_to_ipv4,
    error::P2PError,
    subscriber::events::{BroadcastEvent, ChainEvent},
    types::block::Blocks,
};

pub const MAX_OUTBOUND_CONNECTION_MAILBOX: usize = 1 << 10;
pub const MAX_INBOUND_CONNECTION_MAILBOX: usize = 1 << 9;

lazy_static::lazy_static! {
    pub static ref ZERO_PEER: PeerId =
        PeerId::from_str("QmX5e9hkQf7B45e2MZf38vhsC2wfA5aKQrrBuLujwaUBGw").unwrap();
}

pub type AuthorFn = dyn Fn(Handshake) -> bool + Send + Sync;
pub type HandleMsgFn = dyn Fn(PeerId, RawMessage) -> Result<(), String> + Send + Sync;

pub fn author_handshake(genesis: Hash) -> impl Fn(Handshake) -> bool {
    move |handshake: Handshake| handshake.genesis() == &genesis
}

#[derive(Clone)]
pub struct TcpServerHandle {
    tx: tokio::sync::mpsc::UnboundedSender<ServerEvent>,
}

impl TcpServerHandle {
    pub async fn send(&self, event: ServerEvent) -> Result<Result<PeerId, P2PError>, ()> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(ServerEvent::WithReply(Box::new(event), reply_tx))
            .map_err(|_| ())?;
        reply_rx.await.map_err(|_| ())
    }

    pub fn try_send(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }
}

pub enum ServerEvent {
    Connected(PeerId, BoundType, SessionTx, RawMessage),
    Disconnected(PeerId),
    Message(PeerId, RawMessage),
    Ping(PeerId),
    WithReply(Box<ServerEvent>, tokio::sync::oneshot::Sender<Result<PeerId, P2PError>>),
}

struct ConnectInfo {
    connect_time: chrono::DateTime<chrono::Utc>,
    bound_type: BoundType,
    write_tx: SessionTx,
}

impl ConnectInfo {
    fn new(connect_time: chrono::DateTime<chrono::Utc>, bound_type: BoundType, write_tx: SessionTx) -> Self {
        ConnectInfo {
            connect_time,
            bound_type,
            write_tx,
        }
    }
}

#[derive(Clone)]
pub struct TcpServer {
    node_info: (PeerId, Multiaddr),
    peers: Arc<RwLock<HashMap<PeerId, ConnectInfo>>>,
    genesis: Hash,
    cache: Arc<RwLock<LruCache<Hash, bool>>>,
    author_fn: Arc<AuthorFn>,
    handles: Arc<HandleMsgFn>,
    server_handle: TcpServerHandle,
}

impl TcpServer {
    pub fn new(
        peer_id: PeerId,
        mul_addr: Multiaddr,
        _key: Option<()>,
        genesis: Hash,
        author: Box<dyn Fn(Handshake) -> bool + Send + Sync>,
        handles: Box<dyn Fn(PeerId, RawMessage) -> Result<(), String> + Send + Sync>,
    ) -> (Self, TcpServerHandle) {
        let author = Arc::new(author);
        let handles = Arc::new(handles);
        let mut addr: String = String::new();
        for item in mul_addr.iter() {
            match &item {
                Protocol::Ip4(ip4) => addr.push_str(&format!("{}:", ip4)),
                Protocol::Tcp(port) => addr.push_str(&format!("{}", port)),
                _ => {}
            }
        }
        let socket_addr = net::SocketAddr::from_str(&addr).unwrap();

        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<ServerEvent>();

        let server_handle = TcpServerHandle {
            tx: event_tx.clone(),
        };
        let server_handle_for_spawn = server_handle.clone();

        let server = TcpServer {
            node_info: (peer_id, mul_addr.clone()),
            peers: Arc::new(RwLock::new(HashMap::new())),
            genesis,
            cache: Arc::new(RwLock::new(LruCache::with_expiry_duration_and_capacity(Duration::from_secs(5), 100_000))),
            author_fn: author,
            handles,
            server_handle: server_handle.clone(),
        };

        let peers = server.peers.clone();
        let genesis = server.genesis;
        let node_info = server.node_info.clone();
        let author_fn = server.author_fn.clone();
        let handles = server.handles.clone();
        let cache = server.cache.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async move {
                let listener = TcpListener::bind(socket_addr).await.expect("bind");
                let mut peer_cleanup = interval(Duration::from_secs(3));

                loop {
                    tokio::select! {
                        accept = listener.accept() => {
                        if let Ok((stream, _)) = accept {
                            let (read, write) = stream.into_split();
                            let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel();
                            let session = Session::new(
                                *ZERO_PEER,
                                node_info.0,
                                server_handle_for_spawn.clone(),
                                BoundType::InBound,
                                genesis,
                                write_tx,
                            );
                            tokio::spawn(async move {
                                session.run(read, write, write_rx).await;
                            });
                        }
                    }
                    event = event_rx.recv() => {
                        if let Some(ev) = event {
                            match ev {
                                ServerEvent::WithReply(inner, reply) => {
                                    let result = handle_server_event(&inner, &peers, &cache, &author_fn, &handles, &node_info, &genesis);
                                    let _ = reply.send(result);
                                }
                                _ => {
                                    let _ = handle_server_event(&ev, &peers, &cache, &author_fn, &handles, &node_info, &genesis);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    _ = peer_cleanup.tick() => {
                        let now = chrono::Utc::now();
                        let mut to_remove = vec![];
                        for (peer, info) in peers.read().iter() {
                            if (now - info.connect_time).num_seconds() > 3 {
                                to_remove.push(*peer);
                            }
                        }
                        for peer in to_remove {
                            peers.write().remove(&peer);
                        }
                    }
                }
            }
            });
        });

        (server, server_handle)
    }

    pub fn broadcast(&self, msg: &RawMessage) {
        let peers = self.peers.read();
        if let Some(peer_bytes) = &msg.header().peer_id {
            if let Ok(peer_id) = PeerId::from_bytes(peer_bytes.as_slice()) {
                if let Some(info) = peers.get(&peer_id) {
                    let _ = info.write_tx.send(msg.clone());
                }
            }
        } else {
            for info in peers.values() {
                let _ = info.write_tx.send(msg.clone());
            }
        }
    }

    pub fn handle_broadcast_event(&self, event: BroadcastEvent) {
        match event {
            BroadcastEvent::Consensus(msg) => {
                let header = RawHeader::new(P2PMsgCode::Consensus, 10, chrono::Local::now().timestamp_millis() as u64, None);
                let payload = msg.into_payload();
                let raw_msg = RawMessage::new(header, payload);
                self.broadcast(&raw_msg);
            }
            BroadcastEvent::Blocks(peer_id, blocks) => {
                let mut header = RawHeader::new(P2PMsgCode::Block, 10, chrono::Local::now().timestamp_millis() as u64, None);
                if let Some(pid) = peer_id {
                    header.peer_id = Some(pid.to_bytes().to_vec());
                }
                let payload = StorageValue::into_bytes(blocks);
                let raw_msg = RawMessage::new(header, payload);
                self.broadcast(&raw_msg);
            }
            BroadcastEvent::Sync(height) => {
                if let Some(peer_id) = self.peers.read().keys().next().cloned() {
                    let header = RawHeader::new(P2PMsgCode::Sync, 10, chrono::Local::now().timestamp_millis() as u64, Some(peer_id.to_bytes().to_vec()));
                    let payload = height.into_bytes();
                    let raw_msg = RawMessage::new(header, payload);
                    self.broadcast(&raw_msg);
                }
            }
            _ => {}
        }
    }

    pub fn handle_chain_event(&self, event: ChainEvent) {
        match event {
            ChainEvent::NewBlock(block) => {
                self.handle_broadcast_event(BroadcastEvent::Blocks(None, Blocks(vec![block])));
            }
            ChainEvent::SyncBlock(height) => {
                self.handle_broadcast_event(BroadcastEvent::Sync(height));
            }
            ChainEvent::PostBlock(peer_id, blocks) => {
                self.handle_broadcast_event(BroadcastEvent::Blocks(peer_id, blocks));
            }
            _ => {}
        }
    }

    pub fn add_peer(&self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {
        if self.peers.read().contains_key(&remote_id) {
            return;
        }
        let mul_addr = remote_addresses[0].clone();
        let local_id = self.node_info.0;
        let genesis = self.genesis;
        let handle = self.clone_handle();
        tokio::spawn(async move {
            let delay = rand::random::<u64>() % 100;
            sleep(Duration::from_millis(delay)).await;
            if let Ok(socket_addr) = multiaddr_to_ipv4(&mul_addr) {
                if let Ok(stream) = TcpStream::connect(socket_addr).await {
                    let (read, write) = stream.into_split();
                    let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel();
                    let session = Session::new(
                        remote_id,
                        local_id,
                        handle,
                        BoundType::OutBound,
                        genesis,
                        write_tx,
                    );
                    session.run(read, write, write_rx).await;
                }
            }
        });
    }

    fn clone_handle(&self) -> TcpServerHandle {
        self.server_handle.clone()
    }
}

fn handle_server_event(
    event: &ServerEvent,
    peers: &Arc<RwLock<HashMap<PeerId, ConnectInfo>>>,
    cache: &Arc<RwLock<LruCache<Hash, bool>>>,
    author_fn: &Arc<AuthorFn>,
    handles: &Arc<HandleMsgFn>,
    node_info: &(PeerId, Multiaddr),
    genesis: &Hash,
) -> Result<PeerId, P2PError> {
    match event {
        ServerEvent::Connected(_peer_id, bound_type, write_tx, raw_msg) => {
            use std::borrow::Cow;
            let handshake: Handshake = StorageValue::from_bytes(Cow::from(raw_msg.payload().to_vec()));
            let peer_id = handshake.peer_id();
            if peers.read().contains_key(&peer_id) {
                return Err(P2PError::DumpConnected);
            }
            if node_info.0 == handshake.peer_id() {
                return Err(P2PError::HandShakeFailed);
            }
            if !author_fn(handshake) {
                return Err(P2PError::DifferentGenesis);
            }
            let connect_info = ConnectInfo::new(chrono::Utc::now(), *bound_type, write_tx.clone());
            peers.write().insert(peer_id, connect_info);
            Ok(peer_id)
        }
        ServerEvent::Disconnected(peer_id) => {
            peers.write().remove(peer_id);
            Ok(*peer_id)
        }
        ServerEvent::Ping(peer_id) => {
            if let Some(info) = peers.write().get_mut(peer_id) {
                info.connect_time = chrono::Utc::now();
            }
            Ok(*peer_id)
        }
        ServerEvent::Message(peer_id, raw_msg) => {
            let hash: Hash = raw_msg.hash();
            let now = chrono::Local::now().timestamp_millis() as u64;
            if now < raw_msg.header().create_time {
                return Ok(*peer_id);
            }
            {
                let mut cache_guard = cache.write();
                if cache_guard.get(&hash).is_some() {
                    return Ok(*peer_id);
                }
                cache_guard.insert(hash, true);
            }
            let _ = handles(*peer_id, raw_msg.clone());
            Ok(*peer_id)
        }
        ServerEvent::WithReply(inner, _) => handle_server_event(inner, peers, cache, author_fn, handles, node_info, genesis),
    }
}
