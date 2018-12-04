use std::collections::HashMap;
use std::net;
use std::str::FromStr;

use std::any::{Any, TypeId};

use tokio::{codec::FramedRead, io::AsyncRead, io::WriteHalf, net::TcpListener, net::TcpStream};
use actix::prelude::*;
use futures::prelude::*;
use uuid::Uuid;
use libp2p::{
    core::upgrade::{self, OutboundUpgradeExt},
    multiaddr::Protocol,
    Transport,
    secio,
    mplex,
    core::nodes::swarm::NetworkBehaviour,
    floodsub::FloodsubMessage,
    PeerId,
    Multiaddr,
};

use crate::{
    common::multiaddr_to_ipv4,
    subscriber::P2PEvent,
    util::{TimerOp, TimerRuntime},
};
use super::codec::MsgPacketCodec;
use super::session::{self, Session};

pub const MAX_OUTBOUND_CONNECTION_MAILBOX: usize = 1 << 10;
pub const MAX_INBOUND_CONNECTION_MAILBOX: usize = 1 << 9;

#[derive(Message)]
pub enum ServerEvent {
    Connected(PeerId),
    Disconnected(PeerId),
}

pub struct TcpServer {
    pid: Addr<TcpServer>,
    key: Option<secio::SecioKeyPair>,
    node_info: (PeerId, Multiaddr),
    peers: HashMap<PeerId, u8>,
    dial_peers: HashMap<PeerId, Uuid>,
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("[{:?}] Server start, listen on: {:?}", self.node_info.0, self.node_info.1);
        ctx.run_interval(::std::time::Duration::from_secs(2), move |act, ctx| {
            let peers: String = act.peers.keys().map(|key| key.to_base58()).collect::<Vec<String>>().join(",");
            info!("Connect clients: {}, [{}]", act.peers.len(), peers);
            let dia_peers: String = act.dial_peers.keys().map(|key| key.to_base58()).collect::<Vec<String>>().join(",");
            info!("Dialing clients: {}, [{}]", act.dial_peers.len(), dia_peers);
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("[{:?}] Server stopped, listen on: {:?}", self.node_info.0, self.node_info.1);
    }
}

impl Handler<P2PEvent> for TcpServer {
    type Result = ();

    /// handle p2p event
    fn handle(&mut self, msg: P2PEvent, ctx: &mut Self::Context) -> Self::Result {
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
    type Result = ();
    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServerEvent::Connected(ref peer_id) => {
                trace!("Connected peer: {:?}", peer_id);
                if self.peers.contains_key(peer_id) {
                    unimplemented!("It should be happen");
                    return ();
                }
                self.peers.entry(peer_id.clone()).or_insert(0);
                self.dial_peers.remove(peer_id);
            }
            ServerEvent::Disconnected(ref peer_id) => {
                trace!("Disconnected peer: {:?}", peer_id);
                self.peers.remove(&peer_id);
                self.dial_peers.remove(peer_id);
            }
        }
        ()
    }
}

impl TcpServer {
    pub fn new(peer_id: PeerId, mul_addr: Multiaddr, key: Option<secio::SecioKeyPair>) -> Addr<TcpServer> {
        let mut addr: String = "".to_string();
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
            let peer_id_c = peer_id.clone();
            ctx.add_message_stream(
                lis.incoming()
                    .map_err(|_| ())
                    .map(move |s| {
                        let peer_id_c = peer_id_c.clone();
                        // There have a problem
                        TcpConnect(s, peer_id_c)
                    }),
            );
            TcpServer {
                pid: ctx.address().clone(),
                key: key,
                node_info: (peer_id.clone(), mul_addr.clone()),
                peers: HashMap::new(),
                dial_peers: HashMap::new(),
            }
        })
    }

    fn add_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, chrono::Local::now().to_string().as_bytes());
        if self.dial_peers.contains_key(&remote_id) || self.peers.contains_key(&remote_id) {
            return;
        }

        let result = self.dial_peers.entry(remote_id.clone()).or_insert(id.clone());
        assert_eq!(*result, id);

        // try to connect, dial it
        TcpDial::new(remote_id, remote_addresses[0].clone(), self.pid.clone());
    }

    // TODO
    fn drop_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {}
}

#[derive(Message)]
struct TcpConnect(TcpStream, PeerId);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, ctx: &mut Context<Self>) {
        trace!("TcpServer receive tcp connect event, peerid: {:?}", msg.1);
        // For each incoming connection we create `session` actor with out chat server
        if self.dial_peers.contains_key(&msg.1) {
            msg.0.shutdown(net::Shutdown::Both);
            return;
        }
        if self.peers.contains_key(&msg.1) {
            msg.0.shutdown(net::Shutdown::Both);
            return;
        }

        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, chrono::Local::now().to_string().as_bytes());
        let got_id = self.dial_peers.entry(msg.1.clone()).or_insert(id.clone());
        assert_eq!(*got_id, id);

        let peer_id = msg.1.clone();
        let server_id = self.pid.clone();
        Session::create(|ctx| {
            let (r, w) = msg.0.split();
            Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
            Session::new(
                peer_id,
                server_id,
                actix::io::FramedWrite::new(w, MsgPacketCodec, ctx),
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
    pub fn new(peer_id: PeerId, mul_addr: Multiaddr, tcp_server: Addr<TcpServer>) {
        let socket_addr = multiaddr_to_ipv4(&mul_addr).unwrap();
        trace!("Try to dial remote peer, peer_id:{:?}, network: {:?}", &peer_id, &socket_addr);
        Arbiter::spawn(TcpStream::connect(&socket_addr).and_then(move |stream| {
            trace!("Dialing remote peer: {:?}", peer_id);
            let peer_id = peer_id.clone();
            let tcp_server = tcp_server.clone();
            Session::create(|ctx| {
                let (r, w) = stream.split();
                Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
                Session::new(peer_id, tcp_server, actix::io::FramedWrite::new(w, MsgPacketCodec, ctx))
            });

            futures::future::ok(())
        }).map_err(|e| {
            error!("Dial tcp connect fail, err: {}", e);
            ()
        }));
    }
}