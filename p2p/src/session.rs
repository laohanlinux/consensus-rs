use actix::prelude::*;
use std::io::{self, Write};
use std::net;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use codec::{Request, Response, RequestPayload, ResponsePayload,
            RawCodec, RawMessage, P2PMessage, TId, TAddr, TValue};

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
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, RawCodec>,
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
impl StreamHandler<RawMessage<TId, TAddr, TValue>, io::Error> for Session {
    /// This is main event loop for client requests
    fn handle(&mut self, msg: io::Result<Option<RawMessage<TId, TAddr, TValue>>>, ctx: &mut Self::Context) {
        let msg = msg.unwrap();
        writeln!(io::stdout(), "session receive client request msg").unwrap();
        if msg.is_none() {
            return
        }

        let msg = msg.unwrap();
        match msg {
            RawMessage::P2P(p2p_msg) =>{
//                self.proccess_raw_msg(p2p_msg,&mut ctx);
            },
            _ => unimplemented!()
        }
    }
}

impl Session {
    pub fn new(
        node: Node<TId, TAddr>,
        addr: Addr<Server>,
        framed: actix::io::FramedWrite<WriteHalf<TcpStream>, RawCodec>,
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
            let pong = RawMessage::new_p2p_response(pong);
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
        let ping = RequestType{
            caller: self.node.clone(),
            request_id: u64::gen(64),
            payload: RequestPayload::Ping,
        };

        ctx.run_later(Duration::new(1,0), move |act, ctx| {
            writeln!(io::stdout(), "client send a ping").unwrap();
            let raw_message = RawMessage::new_p2p_request(ping);
            act.framed.write(raw_message);
            act.hb(ctx);
        });
    }

    fn proccess_raw_msg(&self, msg: P2PMessage<TId, TAddr, TValue>,  ctx: &mut Context<Self>) {
        match msg {
            P2PMessage::Req(request) => {
                self.addr.do_send(request);
            },
            P2PMessage::Resp(response) =>{
                // TODO
                //self.addr.do_send(response);
                unimplemented!()
            },
        }
    }
}

impl Handler<Message> for Session {
    type Result = ();
    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
    }
}