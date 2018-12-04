use std::io;
use std::net;
use std::str::FromStr;
use std::time::Duration;

use futures::Future;
use futures::Stream;
use actix::prelude::*;
use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;
use libp2p::PeerId;
use tokio::{codec::FramedRead, io::AsyncRead, io::WriteHalf, net::TcpListener, net::TcpStream};

use crate::{
    common::multiaddr_to_ipv4,
};
use super::codec::MsgPacketCodec;
use super::protocol::RawMessage;
use super::server::{TcpServer, ServerEvent};

pub struct Session {
    peer_id: PeerId,
    server: Addr<TcpServer>,
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(ServerEvent::Connected(self.peer_id.clone()));
        trace!("P2P session created");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(ServerEvent::Disconnected(self.peer_id.clone()));
        trace!("P2P session stopped");
    }
}

impl actix::io::WriteHandler<io::Error> for Session {}

/// receive raw message from network, forward it to server
impl StreamHandler<RawMessage, io::Error> for Session {
    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        // forward message to server handle
//        self.server.do_send(msg);
    }
}

/// receive raw message from server, forward it to network
impl Handler<RawMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        self.framed.write(msg);
    }
}

impl Session {
    pub fn new(
        self_peer_id: PeerId,
        server: Addr<TcpServer>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
    ) -> Session {
        Session {
            peer_id: self_peer_id,
            server: server,
            framed: framed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Server;

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