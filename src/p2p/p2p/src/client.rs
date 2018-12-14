use actix::prelude::*;
use futures::Future;
use std::str::FromStr;
use std::time::Duration;
use std::{io,io::Write, net, process, thread};
use tokio_io::codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;

use codec::{Codec, OutboundCode, Request, RequestPayload, Response, ResponsePayload, TId, TAddr, TValue};
use kad::base::Node;

pub struct Client {
    pub node: Node<TId, TAddr>,
    pub framed: actix::io::FramedWrite<WriteHalf<TcpStream>, OutboundCode>,
}

#[derive(Message)]
struct ClientCommand(String);

impl Actor for Client {
    type Context = Context<Self>;
    fn started(&mut self, ctx:&mut Context<Self>) {
        writeln!(io::stdout(), "client started...").unwrap();
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) ->Running {
        println!("Disconnectd");
        System::current().stop();
        Running::Stop
    }
}

impl Client {
    fn hb(&self, ctx: &mut Context<Self>) {
        let node = self.node.clone();
        ctx.run_later(Duration::new(1,0), move |act, ctx| {
            writeln!(io::stdout(), "client send a ping").unwrap();
            let req = Request::new(node, 100, RequestPayload::Ping);
            act.framed.write(req);
            act.hb(ctx);
        });
    }
}

impl Handler<Request<TId, TAddr, TValue>> for Client {
    type Result = Response<TId, TAddr, TValue>;

    fn handle(&mut self, msg: Request<TId, TAddr, TValue>, ctx: &mut Context<Self>) -> Response<TId, TAddr, TValue> {
        let node = self.node.clone();
        let req = Request::new(node.clone(), 100, RequestPayload::Ping);
        self.framed.write(msg);
        let resp = Response::new(req, node, ResponsePayload::NoResult);

        resp
    }
}

impl actix::io::WriteHandler<io::Error> for Client {}

/// Server communication
impl StreamHandler<Response<TId, TAddr, TValue>, io::Error> for Client {
    fn handle(
        &mut self, msg: io::Result<Option<Response<TId, TAddr, TValue>>>, ctx: &mut Context<Self>,
    ) {
        println!("receive server msg: {:?}", msg);
//        match msg {
//            Ok(Some(codec::ChatResponse::Message(ref msg))) => {
//                println!("message: {}", msg);
//            }
//            Ok(Some(codec::ChatResponse::Joined(ref msg))) => {
//                println!("!!! joined: {}", msg);
//            }
//            Ok(Some(codec::ChatResponse::Rooms(rooms))) => {
//                println!("\n!!! Available rooms:");
//                for room in rooms {
//                    println!("{}", room);
//                }
//                println!();
//            }
//            _ => ctx.stop(),
//        }
    }
}