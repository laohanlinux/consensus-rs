use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::net;
use std::str::FromStr;
use std::time::{Duration, Instant};

use actix::prelude::*;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::Hash;
use futures::prelude::*;
use libp2p::{
    core::nodes::swarm::NetworkBehaviour,
    core::upgrade::{self, OutboundUpgradeExt},
    floodsub::FloodsubMessage,
    mplex,
    multiaddr::Protocol,
    secio, Multiaddr, PeerId, Transport,
};
use tokio::{timer::Delay, codec::FramedRead, io::AsyncRead, io::WriteHalf, net::TcpListener, net::TcpStream};
use uuid::Uuid;

use super::codec::MsgPacketCodec;
use super::protocol::{BoundType, RawMessage, P2PMsgCode, Handshake};
use super::session::Session;
use crate::{
    common::{multiaddr_to_ipv4, random_uuid},
    error::P2PError,
    subscriber::P2PEvent,
};

pub const MAX_OUTBOUND_CONNECTION_MAILBOX: usize = 1 << 10;
pub const MAX_INBOUND_CONNECTION_MAILBOX: usize = 1 << 9;

lazy_static! {
    pub static ref ZERO_PEER: PeerId =
        { PeerId::from_str("QmX5e9hkQf7B45e2MZf38vhsC2wfA5aKQrrBuLujwaUBGw").unwrap() };
}

pub type author_fn = Fn(Handshake) -> bool;
pub type handshake_packet_fn = Fn() -> Handshake;

pub fn author_handshake(genesis: Hash) -> impl Fn(Handshake) -> bool {
    let author = move |handshake: Handshake| {
        if *handshake.genesis() != genesis {
            return false;
        }
        true
    };
    author
}

pub enum ServerEvent {
    Connected(PeerId, BoundType, RawMessage),
    // handshake
    Disconnected(PeerId),
    Message(RawMessage),
}

impl Message for ServerEvent {
    type Result = Result<PeerId, P2PError>;
}

pub struct TcpServer {
    pid: Addr<TcpServer>,
    key: Option<secio::SecioKeyPair>,
    node_info: (PeerId, Multiaddr),
    peers: HashMap<PeerId, ConnectInfo>,
    genesis: Hash,
    author_fn: Box<author_fn>,
}

struct ConnectInfo {
    connect_time: chrono::DateTime<chrono::Utc>,
    bound_type: BoundType,
}

impl ConnectInfo {
    fn new(connect_time: chrono::DateTime<chrono::Utc>, bound_type: BoundType) -> Self {
        ConnectInfo {
            connect_time: connect_time,
            bound_type: bound_type,
        }
    }
}

fn node_info(peers: &HashMap<PeerId, ConnectInfo>) -> String {
    let mut info: Vec<String> = vec![];
    for peer in peers {
        info.push(format!(
            "{}----> [bound: {:?}, connect_time: {:?}]",
            peer.0.to_base58(),
            peer.1.bound_type,
            peer.1.connect_time
        ));
    }
    info.join("\n")
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "[{:?}] Server start, listen on: {:?}",
            self.node_info.0, self.node_info.1
        );
        ctx.run_interval(::std::time::Duration::from_secs(2), |act, _| {
            info!(
                "Connect clients: {}\nlocal-id:{}, \n{}",
                act.peers.len(),
                act.node_info.0.to_base58(),
                node_info(&act.peers)
            );
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!(
            "[{:?}] Server stopped, listen on: {:?}",
            self.node_info.0.to_base58(),
            self.node_info.1
        );
    }
}

impl Handler<P2PEvent> for TcpServer {
    type Result = ();

    /// handle p2p event
    fn handle(&mut self, msg: P2PEvent, _: &mut Self::Context) -> Self::Result {
        match msg {
            P2PEvent::AddPeer(remote_peer, remote_addresses) => {
                self.add_peer(remote_peer, remote_addresses);
            }
            P2PEvent::DropPeer(remote_peer, remote_addresses) => {
                self.drop_peer(remote_peer, remote_addresses);
            }
        }
        ()
    }
}

impl Handler<ServerEvent> for TcpServer {
    type Result = Result<PeerId, P2PError>;
    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServerEvent::Connected(ref peer_id, ref bound_type, ref raw_msg) => {
                trace!("Connected peer: {:?}", peer_id);
                return self.handle_handshake(bound_type.clone(), raw_msg.payload());
            }
            ServerEvent::Disconnected(ref peer_id) => {
                trace!("Disconnected peer: {:?}", peer_id);
                self.peers.remove(&peer_id);
            }
            ServerEvent::Message(ref raw_msg) => {
//                self.handle_network_message(raw_msg.clone())?;
            }
        }
        Err(P2PError::InvalidMessage)
    }
}

impl TcpServer {
    pub fn new(
        peer_id: PeerId,
        mul_addr: Multiaddr,
        key: Option<secio::SecioKeyPair>,
        genesis: Hash,
        author: Box<Fn(Handshake) -> bool>,
    ) -> Addr<TcpServer> {
        let mut addr: String = String::new();
        mul_addr.iter().for_each(|item| match &item {
            Protocol::Ip4(ref ip4) => {
                addr.push_str(&format!("{}:", ip4));
            }
            Protocol::Tcp(ref port) => {
                addr.push_str(&format!("{}", port));
            }
            _ => {}
        });
        let socket_addr = net::SocketAddr::from_str(&addr).unwrap();

// bind tcp listen address
        let lis = TcpListener::bind(&socket_addr).unwrap();
// create tcp server and dispatch coming connection to self handle
        TcpServer::create(move |ctx| {
            ctx.set_mailbox_capacity(MAX_INBOUND_CONNECTION_MAILBOX);
            ctx.add_message_stream(lis.incoming().map_err(|_| ()).map(move |s| {
                info!("New connection are comming");
                TcpConnectInBound(s)
            }));
            TcpServer {
                pid: ctx.address().clone(),
                key: key,
                node_info: (peer_id.clone(), mul_addr.clone()),
                peers: HashMap::new(),
                genesis: genesis,
                author_fn: author,
            }
        })
    }

    fn add_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {
        if self.peers.contains_key(&remote_id) {
            return;
        }

        let mul_addr = remote_addresses[0].clone();
        let local_id = self.node_info.0.clone();
        let server_id = self.pid.clone();
        let genesis = self.genesis.clone();
        let delay = rand::random::<u64>() % 100;
        let timer_fut = Delay::new(Instant::now() + Duration::from_millis(delay));
        tokio::spawn(timer_fut.and_then(move |_| {
        // try to connect, dial it
            TcpDial::new(
                remote_id,
                local_id,
                mul_addr,
                genesis,
                server_id,
            );
            futures::future::ok(())
        }).map_err(|err| panic!(err)));
    }

    // TODO
    fn drop_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {}

    fn handle_network_message(&mut self, msg: RawMessage) -> Result<(), P2PError> {
        let header = msg.header();
        match header.code {
            P2PMsgCode::Handshake => {
                return Err(P2PError::InvalidMessage);
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    fn handle_handshake(
        &mut self,
        bound_type: BoundType,
        payload: &Vec<u8>,
    ) -> Result<PeerId, P2PError> {
        use std::borrow::Cow;
        let handshake: Handshake = Handshake::from_bytes(Cow::from(payload));
        let peer_id = handshake.peer_id();
        if self.peers.contains_key(&peer_id) {
            return Err(P2PError::DumpConnected);
        }
        if self.node_info.0 == handshake.peer_id() {
            return Err(P2PError::HandShakeFailed);
        }

        if !(self.author_fn)(handshake.clone()) {
            return Err(P2PError::DifferentGenesis);
        }

        match bound_type {
            BoundType::InBound => {}
            BoundType::OutBound => {}
        }
        let connect_info = ConnectInfo::new(chrono::Utc::now(), BoundType::InBound);
        self.peers.entry(peer_id.clone()).or_insert(connect_info);
        Ok(peer_id)
    }
}

#[derive(Message)]
struct TcpConnectOutBound(TcpStream, PeerId);

/// Handle stream of TcpStream's
impl Handler<TcpConnectOutBound> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnectOutBound, ctx: &mut Context<Self>) {
        trace!("TcpServer receive tcp connect event, peerid: {:?}", msg.1);
// For each incoming connection we create `session` actor with out chat server
        if self.peers.contains_key(&msg.1) {
            msg.0.shutdown(net::Shutdown::Both).unwrap();
            return;
        }

        let peer_id = msg.1.clone();
        let server_id = self.pid.clone();
        let local_id = self.node_info.0.clone();
        let genesis = self.genesis.clone();
        Session::create(move |ctx| {
            let (r, w) = msg.0.split();
            Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
            Session::new(
                ctx.address().clone(),
                peer_id,
                local_id,
                server_id,
                actix::io::FramedWrite::new(w, MsgPacketCodec, ctx),
                BoundType::OutBound,
                genesis,
            )
        });
    }
}

#[derive(Message)]
struct TcpConnectInBound(TcpStream);

impl Handler<TcpConnectInBound> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnectInBound, _: &mut Context<Self>) {
        let server_id = self.pid.clone();
        let local_id = self.node_info.0.clone();
        let genesis = self.genesis.clone();
        Session::create(move |ctx| {
            let (r, w) = msg.0.split();
            Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
            Session::new(
                ctx.address().clone(),
                ZERO_PEER.clone(),
                local_id,
                server_id,
                actix::io::FramedWrite::new(w, MsgPacketCodec, ctx),
                BoundType::InBound,
                genesis,
            )
        });
    }
}

pub struct TcpDial {
    server: Addr<TcpServer>,
}

impl Actor for TcpDial {
    type Context = Context<Self>;
}

impl TcpDial {
    pub fn new(
        peer_id: PeerId,
        local_id: PeerId,
        mul_addr: Multiaddr,
        genesis: Hash,
        tcp_server: Addr<TcpServer>,
    ) {
        let socket_addr = multiaddr_to_ipv4(&mul_addr).unwrap();
        trace!(
            "Try to dial remote peer, peer_id:{:?}, network: {:?}",
            &peer_id,
            &socket_addr
        );
        Arbiter::spawn(
            TcpStream::connect(&socket_addr)
                .and_then(move |stream| {
                    trace!("Dialing remote peer: {:?}", peer_id);
                    let peer_id = peer_id.clone();
                    let local_id = local_id.clone();
                    let genesis = genesis.clone();
                    let tcp_server = tcp_server.clone();
                    Session::create(move |ctx| {
                        let (r, w) = stream.split();
                        Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
                        Session::new(
                            ctx.address().clone(),
                            peer_id,
                            local_id,
                            tcp_server,
                            actix::io::FramedWrite::new(w, MsgPacketCodec, ctx),
                            BoundType::OutBound,
                            genesis,
                        )
                    });

                    futures::future::ok(())
                })
                .map_err(|e| {
                    error!("Dial tcp connect fail, err: {}", e);
                    ()
                }),
        );
    }
}
