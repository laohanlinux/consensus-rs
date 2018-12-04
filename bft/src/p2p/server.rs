use std::collections::HashMap;

use std::any::{Any, TypeId};
use actix::prelude::*;
use futures::prelude::*;
use uuid::Uuid;
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
    Connected(PeerId),
    Disconnected(PeerId),
}

pub struct Server {
    pid: Option<Addr<Server>>,
    peer_id: PeerId,
    listen_addr: Multiaddr,
    key: Option<secio::SecioKeyPair>,
    peers: HashMap<PeerId, u8>,
    dial_peers: HashMap<PeerId, Uuid>,
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.listen();
        info!("[{:?}] Server start, listen on: {:?}", self.peer_id, self.listen_addr);
        ctx.run_interval(::std::time::Duration::from_secs(2), move |act, ctx| {
            let peers: String = act.peers.keys().map(|key| key.to_base58()).collect::<Vec<String>>().join(",");
            info!("Connect clients: {}, [{}]", act.peers.len(), peers);
            let dia_peers: String = act.dial_peers.keys().map(|key| key.to_base58()).collect::<Vec<String>>().join(",");
            info!("Dialing clients: {}, [{}]", act.dial_peers.len(), dia_peers);
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("[{:?}] Server stopped, listen on: {:?}", self.peer_id, self.listen_addr)
    }
}

impl Handler<ServerEvent> for Server {
    type Result = ();
    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServerEvent::Connected(ref peer_id) => {
                trace!("Connected peer: {:?}", peer_id);
                if self.peers.contains_key(peer_id) {
                    warn!("It should be happen");
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

impl Handler<P2PEvent> for Server {
    type Result = ();

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

impl Server {
    pub fn new(pid: Option<Addr<Server>>, peer_id: PeerId, listen: Multiaddr, key: Option<secio::SecioKeyPair>) -> Self {
        Server {
            pid,
            peer_id,
            listen_addr: listen,
            key,
            peers: HashMap::new(),
            dial_peers: HashMap::new(),
        }
    }

    fn listen(&mut self) {
        // start tcp server
        session::TcpServer::new(self.peer_id.clone(), self.listen_addr.clone(), self.pid.as_ref().unwrap().clone());
        trace!("Start listen on: {:?}", self.listen_addr);
    }

    fn add_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, chrono::Local::now().to_string().as_bytes());
        if *self.dial_peers.entry(remote_id.clone()).or_insert(id) == id {
            return;
        }

        if self.peers.contains_key(&remote_id) {
            return;
        }
        // try to connect, dial it
        TcpDial::new(remote_id, remote_addresses[0].clone(), self.pid.as_ref().unwrap().clone());
    }

    // TODO
    fn drop_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {}
}