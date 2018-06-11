use actix::prelude::*;
use std::io;
use std::net;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use codec::{Request, Response, RequestPayload, ResponsePayload, Codec};

use server::{Server, Disconnect};
use kad::base::Node;

type RequestType = Request<u64, net::SocketAddr, Vec<u8>>;
type ResponseType = Response<u64, net::SocketAddr, Vec<u8>>;

#[derive(Message)]
pub struct Message(String);

pub struct Session {
    /// id
    id: u64,

    /// this is address of chat server
    addr: Addr<Unsync, Server>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection
    hb: Instant,
    /// Framed wrapper
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, Codec>,
}

impl Actor for Session {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        /// notify server
        self.addr.do_send(Disconnect{id: self.id});
        Running::Stop
    }
}

impl actix::io::WriteHandler<io::Error> for Session{}

/// To use `Framed` with an actor, we have to implement `StreamHandler` trait
impl StreamHandler<RequestType, io::Error> for Session {
    /// This is main event loop for client requests
    fn handle(&mut self, msg: io::Result<Option<RequestType>>, ctx: &mut Self::Context) {
        match msg {
            Ok(req) =>{
                // TODO
                if req.RequestPayload == RequestPayload::Ping {
                    self.hb = Instant::now();
//                    let resp = ResponseType {
//
//                    }
//                    self.framed.write(Response{})
                }
                // TODO check the return value
                self.addr.do_send(req);
            }
            Err(_) => {},
        }
    }
}

impl Session {
    pub fn new(
        id: u64,
        addr: Addr<Unsync, Server>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, Codec>,
    ) -> Session {
        Session {
            id,
            addr,
            framed,
            hb: Instant::now(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method check heartbeats from client
    fn hb(&self, ctx: &mut actix::Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx|{
            // check client heartbeats from client
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                // heartbeat timed out
                println!("Client heartbeat failed, disconnecting!");
                // stop actor
                ctx.stop();
            }
            let node = Node {
                id: self.id,
                address: "127.0.0.1:8080".parse().unwrap(),
            };
            let pong = ResponseType{
                responder: node,
                payload: ResponsePayload::NoResult,
            };
            act.framed.write(pong);
            act.hb(ctx);
        });
    }
}