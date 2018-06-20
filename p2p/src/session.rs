use actix::prelude::*;
use std::io::{self, Write};
use std::net;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use codec::{Request, Response, RequestPayload, ResponsePayload, Codec, TId, TAddr, TValue};

use server::{Server, Disconnect};
use kad::base::{GenericId, Node};

type RequestType = Request<TId, TAddr, TValue>;
type ResponseType = Response<TId, TAddr, TValue>;

#[derive(Message)]
pub struct Message(pub String);

pub struct Session {
    // local node id
    node: Node<TId, TAddr>,

    /// this is address of chat server
    addr: Addr<Server>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection
    hb: Instant,
    /// Framed wrapper
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, Codec>,
}

impl Actor for Session {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        writeln!(io::stdout(), "started a session actor, node: {:?}", self.node).unwrap();
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        /// notify server
        self.addr.do_send(Disconnect{id: self.node.id});
        Running::Stop
    }
}

impl actix::io::WriteHandler<io::Error> for Session{}

/// To use `Framed` with an actor, we have to implement `StreamHandler` trait
impl StreamHandler<RequestType, io::Error> for Session {
    /// This is main event loop for client requests
    fn handle(&mut self, msg: io::Result<Option<RequestType>>, ctx: &mut Self::Context) {
        writeln!(io::stdout(), "session receive client request msg").unwrap();
        let msg = msg.unwrap();
        if let Some(ref req) = msg {
            match req.payload {
                RequestPayload::Ping => {self.hb = Instant::now();},
                _ => {},
            }
        }
        // forward msg to server
            self.addr.do_send(msg.unwrap());
    }
}

impl StreamHandler<ResponseType, io::Error> for Session {
    fn handle(&mut self, msg: io::Result<Option<ResponseType>>, ctx: &mut Self::Context) {
        writeln!(io::stdout(), "session receive client response msg, discard it").unwrap();
    }
}



impl Session {
    pub fn new(
        node: Node<TId, TAddr>,
        addr: Addr<Server>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, Codec>,
    ) -> Session {
        Session {
            node,
            addr,
            framed,
            hb: Instant::now(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method check heartbeats from client
    fn hb(&self, ctx: &mut actix::Context<Self>) {
        // TODO
        let node = self.node.clone();
        ctx.run_later(Duration::new(1, 0), move |act, ctx|{
            let pong = ResponseType{
                request: Request::new(node.clone(), u64::gen(64), RequestPayload::Ping),
                responder: node.clone(),
                payload: ResponsePayload::NoResult,
            };
            // check client heartbeats from client
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                // heartbeat timed out
                println!("Client heartbeat failed, disconnecting!");
                // stop actor
                ctx.stop();
            }
            act.framed.write(pong);
            act.hb(ctx);
        });
    }

    /// send a heartbeat message as client
    fn send_hb(&self, ctx: &mut Context<Self>) {
        let node = self.node.clone();
//        ctx.run_later(Duration::new(1,0), move |act, ctx| {
//            writeln!(io::stdout(), "client send a ping").unwrap();
//            let req:Request<TId, TAddr, TValue> = Request::new(node, 100, RequestPayload::Ping);
//            act.framed.write(req);
//            act.hb(ctx);
//        });
    }
}

impl Handler<Message> for Session {
    type Result = ();
    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
    }
}