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
use super::server::Server;

pub const MAX_OUTBOUND_CONNECTION_MAILBOX: usize = 1 << 10;
pub const MAX_INBOUND_CONNECTION_MAILBOX: usize = 1 << 9;

pub struct Session {
    id: PeerId,
    addr: Addr<Server>,
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("P2P session created");
    }
}

impl actix::io::WriteHandler<io::Error> for Session {}

/// receive raw message from network, forward it to server
impl StreamHandler<RawMessage, io::Error> for Session {
    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        // forward message to server handle
        let addr = ctx.address();
        addr.do_send(msg);
    }
}

/// receive raw message from inner bussiness, forward it to network
impl Handler<RawMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: RawMessage, ctx: &mut Context<Self>) {
        self.framed.write(msg);
    }
}

impl Session {
    pub fn new(
        self_peer_id: PeerId,
        addr: Addr<Server>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, MsgPacketCodec>,
    ) -> Session {
        Session {
            id: self_peer_id,
            addr: addr,
            framed: framed,
        }
    }
}

pub struct TcpServer {
    server: Addr<Server>,
}

impl Actor for TcpServer {
    type Context = Context<Self>;
}

impl TcpServer {
    pub fn new(peer_id: PeerId, mul_addr: Multiaddr, server: Addr<Server>) -> Addr<TcpServer> {
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
            ctx.add_message_stream(
                lis.incoming()
                    .map_err(|_| ())
                    .map(move |s| {
                        let peer_id = peer_id.clone();
                        TcpConnect(s, peer_id)
                    }),
            );
            TcpServer { server }
        })
    }
}


pub struct TcpDial {
    server: Addr<Server>,
}

impl Actor for TcpDial {
    type Context = Context<Self>;
}

impl TcpDial {
    pub fn new(peer_id: PeerId, mul_addr: Multiaddr, server: Addr<Server>) {
        let socket_addr = multiaddr_to_ipv4(&mul_addr).unwrap();
        Arbiter::spawn(TcpStream::connect(&socket_addr).and_then(move |stream|{
            let peer_id = peer_id.clone();
            TcpServer::create( |ctx| {
                ctx.set_mailbox_capacity(MAX_OUTBOUND_CONNECTION_MAILBOX);
                let request = ctx.address().send(TcpConnect(stream, peer_id)).wait().unwrap();
                TcpServer{server}
            });
            futures::future::ok(())
        }).map_err(|e|{
            error!("Dial tcp connect fail, err: {}", e);
            ()
        }));
    }
}

#[derive(Message)]
struct TcpConnect(TcpStream, PeerId);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        // For each incoming connection we create `session` actor with out chat server
        let server = self.server.clone();
        Session::create(|ctx| {
            let (r, w) = msg.0.split();
            Session::add_stream(FramedRead::new(r, MsgPacketCodec), ctx);
            Session::new(
                msg.1,
                server,
                actix::io::FramedWrite::new(w, MsgPacketCodec, ctx),
            )
        });
    }
}
