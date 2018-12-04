use std::borrow::Cow;
use std::io;
use std::net;
use std::str::FromStr;
use std::time::Duration;
use std::sync::mpsc::sync_channel;

use actix::prelude::*;
use cryptocurrency_kit::storage::values::StorageValue;
use futures::stream::once;
use futures::Future;
use futures::Stream;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use libp2p::PeerId;
use tokio::{codec::FramedRead, io::AsyncRead, io::WriteHalf, net::TcpListener, net::TcpStream};

use super::codec::MsgPacketCodec;
use super::protocol::{BoundType, Handshake, Header, P2PMsgCode, RawMessage};
use super::server::{ServerEvent, TcpServer};
use crate::common::multiaddr_to_ipv4;
use crate::error::P2PError;

pub struct Session {
    pid: Option<Addr<Session>>,
    peer_id: PeerId,
    local_id: PeerId,
    server: Addr<TcpServer>,
    bound_type: BoundType,
    handshaked: bool,
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // send a handshake message
        match self.bound_type {
            BoundType::InBound => {}
            BoundType::OutBound => {
                let peer_id = self.local_id.clone();
                let handshake = Handshake::new("0.1.1".to_string(), peer_id.clone());
                info!("Packet-->, peer_id: {:?}", peer_id);
                let raw_message = RawMessage::new(
                    Header::new(
                        P2PMsgCode::Handshake,
                        10,
                        chrono::Local::now().timestamp_nanos() as u64,
                    ),
                    handshake.into_bytes(),
                );
                ctx.add_message_stream(once(Ok(raw_message)));
            }
        }

//        ctx.run_later(Duration::from_secs(1), |act, ctx| {
//            // pass 1s, not receive handshake packet, close the session
//            if act.handshaked {
//                return;
//            }
//            trace!("Handshake timeout, {},  local_id: {}, peer: {}", act.handshaked, act.local_id.to_base58(), act.peer_id.to_base58());
//            act.framed.close();
//            ctx.stop();
//        });
        trace!("P2P session created");
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.server
            .do_send(ServerEvent::Disconnected(self.peer_id.clone()));
        trace!("P2P session stopped, local_id: {}, peer: {}", self.local_id.to_base58(), self.peer_id.to_base58());
        Running::Stop
    }
}

impl actix::io::WriteHandler<io::Error> for Session {}

/// receive raw message from network, forward it to server
impl StreamHandler<RawMessage, io::Error> for Session {
    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        trace!("Read message: {:?}", msg.header());
        match msg.header().code {
            P2PMsgCode::Handshake => {
                if self.handshaked == false {
                    self.server.send(ServerEvent::Connected(self.peer_id.clone(), self.bound_type, msg.clone()))
                        .into_actor(self)
                        .then(move |res, act, ctx| {
                            match res {
                                Ok(res) => {
                                    if let Err(err) = res {
                                        trace!("Author fail, err: {:?}", err);
                                        ctx.stop();
                                    } else {
                                        let peer = res.unwrap();
                                        trace!("Author successfully, before {}, local_id: {}, peer: {}", act.handshaked, act.local_id.to_base58(), act.peer_id.to_base58());
                                        act.handshaked = true;
                                        act.peer_id = peer;
                                        trace!("Author successfully, after {}, local_id: {}, peer: {}", act.handshaked, act.local_id.to_base58(), act.peer_id.to_base58());
                                    }
                                }
                                Err(err) => panic!(err)
                            }
                            actix::fut::ok(())
                        }).wait(ctx);
                }
            }
            P2PMsgCode::Transaction => {}
            P2PMsgCode::Block => {}
            P2PMsgCode::Consensus => {}
            P2PMsgCode::Sync => {}
            _ => ctx.stop(),
        }
    }
}

/// receive raw message from server, forward it to network
impl Handler<RawMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        trace!("Write message: {:?}", msg.header());
        self.framed.write(msg);
    }
}
//
//impl Message for Result<(), P2PError> {
//    type Result = ();
//}
//
//impl Handler<Result<(), P2PError>> for Session {
//    type Result = ();
//
//    fn handle(&mut self, msg: Result<(), P2PError>, ctx: &mut Context<Self>) {

//    }
//}

impl Session {
    pub fn new(
        self_pid: Addr<Session>,
        self_peer_id: PeerId,
        local_peer: PeerId,
        server: Addr<TcpServer>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
        bound_type: BoundType,
    ) -> Session {
        Session {
            pid: Some(self_pid),
            peer_id: self_peer_id,
            local_id: local_peer,
            server: server,
            handshaked: false,
            framed: framed,
            bound_type: bound_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Server;
    use super::*;

    #[test]
    fn t_tcp_server() {
        let peer_id = PeerId::random();
        let mul_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/5678").unwrap();
        println!("{:?}, {:?}", peer_id, mul_addr);
        crate::logger::init_test_env_log();
        let system = System::new("tt");

        let server = Server::create(move |ctx| {
            let pid = ctx.address();
            let peer_id = peer_id.clone();
            Server::new(Some(pid), peer_id, mul_addr, None)
        });
        //        tokio::spawn(futures::lazy(||{
        //            System::current().stop();
        //            futures::future::ok(())
        //        }));
        crate::util::TimerRuntime::new(Duration::from_secs(10));
        system.run();
    }
}
