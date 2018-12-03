use std::collections::HashMap;
use std::any::{Any, TypeId};
use actix::prelude::*;
use futures::prelude::*;
use libp2p::{
    core::upgrade::{self, OutboundUpgradeExt},
    multiaddr::Protocol,
    Transport,
    secio,
    mplex,
    tokio_codec::{FramedRead, LinesCodec},
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
use super::session::{self, Session, TcpServer, TcpDial};

#[derive(Message)]
pub enum ServerEvent {
    Connected(PeerId, Multiaddr),
    Disconnected(PeerId, Multiaddr),
}

pub struct Server {
    pid: Option<Addr<Server>>,
    peer_id: PeerId,
    listen_addr: Multiaddr,
    key: Option<secio::SecioKeyPair>,
    peers: HashMap<PeerId, Vec<Multiaddr>>,
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.listen();
        info!("[{:?}] Server start, listen on: {:?}", self.peer_id, self.listen_addr);
    }
}

impl Handler<ServerEvent> for Server {
    type Result = ();
    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServerEvent::Connected(ref peer_id, ref mul_addr) => {
                trace!("Connected peer: {:?}", peer_id);
                if self.peers.contains_key(peer_id) {
                    warn!("It should be happen");
                    return ();
                }
                self.peers.entry(peer_id.clone()).or_insert(vec![mul_addr.clone()]);
            }
            ServerEvent::Disconnected(ref peer_id, ref mul_addr) => {
                trace!("Disconnected peer: {:?}", peer_id);
                self.peers.remove(&peer_id);
            }
        }
        ()
    }
}

impl Handler<P2PEvent> for Server {
    type Result = ();

    fn handle(&mut self, msg: P2PEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            P2PEvent::AddPeer(remote_peer, remote_addresses) => {
                self.peers.entry(remote_peer).or_insert(remote_addresses);
            }
            P2PEvent::DropPeer(remote_peer, _) => {
                self.peers.remove_entry(&remote_peer);
            }
        }
        ()
    }
}

impl Server {
    pub fn new(pid: Option<Addr<Server>>, peer_id: PeerId, listen: Multiaddr, key: Option<secio::SecioKeyPair>) -> Self {
        Server {
            pid,
            peer_id,
            listen_addr: listen,
            key,
            peers: HashMap::new(),
        }
    }

    fn listen(&mut self) {
        // start tcp server
        session::TcpServer::new(self.peer_id.clone(), self.listen_addr.clone(), self.pid.as_ref().unwrap().clone());
        trace!("Start listen on: {:?}", self.listen_addr);
    }

    fn add_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {
        if self.peers.contains_key(&remote_id) {
            return;
        }
        // try to connect, dial it
        TcpDial::new(remote_id, remote_addresses[0].clone(), self.pid.as_ref().unwrap().clone());
    }

    // TODO
    fn drop_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {}
}