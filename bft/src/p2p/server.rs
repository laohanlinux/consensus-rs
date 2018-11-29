use std::collections::HashMap;

use actix::prelude::*;
use futures::prelude::*;
use libp2p::{
    Transport,
    core::upgrade::{self, OutboundUpgradeExt},
    secio,
    mplex,
    tokio_codec::{FramedRead, LinesCodec},
    PeerId,
    Multiaddr,
};

use crate::subscriber::P2PEvent;

#[derive(Message)]
pub enum ServerEvent {}

pub struct Server {
    pid: Addr<Server>,
    key: secio::SecioKeyPair,
    peer_id: PeerId,
    peers: HashMap<PeerId, Vec<Multiaddr>>,
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<ServerEvent> for Server {
    type Result = ();
    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
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
    fn listen(&mut self) {
//        let transport = libp2p::CommonTransport::new()
//            .with_upgrade(secio::SecioConfig::new(self.key))
//            .and_then(move |out, _| {
//                let peer_id = out.remote_key.into_peer_id();
//                let upgrade = mplex::MplexConfig::new().map_outbound(move |muxer| (peer_id, muxer) );
//                upgrade::apply_outbound(out.stream, upgrade).map_err(|e| e.into_io_error())
//            });
    }

    fn add_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {

    }

    fn drop_peer(&mut self, remote_id: PeerId, remote_addresses: Vec<Multiaddr>) {

    }
}