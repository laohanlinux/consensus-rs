use actix::prelude::*;
use rand::{self, Rng};
use std::collections::{HashMap, HashSet};
use std::net;

use session;
use kad::base::Node;
use kad::protocol::{Request as KadRequest, RequestPayload as KadRequestPayload,
                    Response as KadResponse, ResponsePayload as KadResponse};

/// Message for server communications

/// New chat session is created
pub struct Connect{
    pub node: Node<u64, net::SocketAddr>,
    pub addr: Addr<session::Session, _>,
}

/// Response type for Connect message.
///
/// Chat server returns unique session id
impl actix::Message for Connect {
    type Result = usize;
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: u64,
}

/// Send message to peer
/// if id is zero, send message to special peer, otherwise broadcast message
#[derive(Message)]
pub struct Message{
    /// Id of the client session
    pub id: u64,
    /// Peer message
    pub msg: String,
}

pub struct Server {
    sessions: HashMap<u64, Addr<_,session::Session>>,
}

impl Default for Server {
    fn default() -> Server {
        Server {
            sessions: HashMap::new(),
        }
    }
}

impl Server {
    /// Send message to all nodes
    fn send_message(&self, msg: Message) {
        match msg {
            Message{id:0, msg: msg} => {
                self.sessions.iter().for_each(|(_, addr)|{
                    addr.do_send(msg.to_string());
                });
            },
            Message{id: ref id, msg: msg} => {
                if let Some(addr) = self.sessions.get(id) {
                    addr.do_send(msg.to_string());
                }
            },
        }
    }
}

/// Make actor for `Server`
impl Actor for Server {
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign id to this session
impl Handler<Connect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result{
        println!("new connect is comming...");
        let id = msg.node.id;
        // TODO send a dump connect msg
        if self.sessions.get(&id).is_some() {
            return Ok(());
        }
        // TODO add kad logic
        self.sessions.entry(id).or_insert(msg.addr);
        Ok(())
    }
}

/// Handler for Disconnect message
impl Handler<Disconnect> for Server {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("{} disconncetd", msg.id);
        // TODO add kad logic
        self.sessions.remove(&msg.id);
    }
}

// TODO
/// Handler for Message message, example out logic call

impl Handler<KadRequest<u64, net::SocketAddr, Vec<u8>>> for Server {
    type Result = KadResponse<u64, net::SocketAddr, Vec<u8>>;
    fn handle(&mut self, msg: KadRequest<u64, net::SocketAddr, Vec<u8>>, _: &mut Context<Self>) {
        match KadRequest {
            KadRequest{caller: caller, request_id: rid, payload: KadRequestPayload::Ping} => {
                // TODO
            }
            KadRequest{caller: caller, request_id: rid, payload: KadRequestPayload::FindNode(id)} => {
                // TODO
            }
            KadRequest{caller: caller, request_id: rid, payload: KadRequestPayload::FindValue(_)} => {
                unimplemented!();
            }
            KadRequest{caller: caller, request_id: rid, payload: KadRequestPayload::Store(_, _)} => {
                unimplemented!();
            }
        }
    }
}