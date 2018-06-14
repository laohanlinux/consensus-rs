use actix::prelude::*;
use std::collections::{HashMap};
use std::net;

use session;
use kad::knodetable::*;
use kad::base::Node;
use codec::{Request , RequestPayload, Response, ResponsePayload, TId, TAddr, TValue};

type KadRequest = Request<TId, TAddr, TValue>;
type KadRequestPayload = RequestPayload<TId, TValue>;
type KadResponse = Response<TId, TAddr, TValue>;
type KadResponsePayload = ResponsePayload<TId, TAddr, TValue>;

/// Message for server communications

/// New chat session is created
pub struct Connect{
    pub node: Node<TId, TAddr>,
    pub addr: Addr<Unsync, session::Session>,
}

/// Response type for Connect message.
///
/// Chat server returns unique session id
impl actix::Message for Connect {
    type Result = ();
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: TId,
}

/// Send message to peer
/// if id is zero, send message to special peer, otherwise broadcast message
#[derive(Message)]
pub struct Message{
    /// Id of the client session
    pub id: TId,
    /// Peer message
    pub msg: String,
}

pub struct Server {
    tables: KNodeTable<TId, TAddr>,
    sessions: HashMap<TId, Addr<Unsync,session::Session>>,
}

impl Default for Server {
    fn default() -> Server {
        Server {
            tables: KNodeTable::new(0),
            sessions: HashMap::new(),
        }
    }
}

impl Server {
    pub fn new(node_id: TId) -> Server {
        Server {
            tables: KNodeTable::new(node_id),
            sessions: HashMap::new(),
        }
    }

    /// Send message to all nodes
    pub fn send_message(&self, msg: Message) {
        match msg {
            Message{id:0, msg: msg} => {
                self.sessions.iter().for_each(|(_, addr)|{
                    addr.do_send(session::Message(msg.clone()));
                });
            },
            Message{id: id, msg: msg} => {
                if let Some(addr) = self.sessions.get(&id) {
                    addr.do_send(session::Message(msg.clone()));
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

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>){
        println!("new connect is comming...");
        let id = msg.node.id;
        // TODO send a dump connect msg
        if self.sessions.get(&id).is_some() {
            return ;
        }
        // TODO add kad logic
        self.sessions.entry(id).or_insert(msg.addr);
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

impl Handler<Request<TId, TAddr, TValue>> for Server {
    type Result = KadResponse;

    fn handle(&mut self, msg: Request<TId, TAddr, TValue>, _: &mut Context<Self>) -> Response<TId, TAddr, TValue>{
        println!("{:?}", msg.payload);

        match msg {
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::Ping} => {
                // TODO
                unimplemented!();
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::FindNode(id)} => {
                // TODO
                unimplemented!();
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::FindValue(_)} => {
                unimplemented!();
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::Store(_, _)} => {
                unimplemented!();
            },
        }
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use codec::*;
    use kad::base::Node;
    use kad::base::GenericAPI;
    use kad::base::GenericNodeTable;
    use std::io::{self, Write};

    #[test]
    fn test_server(){
        let id = 100;
        let mut server = super::Server::new(id);
        let index = server.tables.random_id();
        writeln!(io::stdout(), "random id {}", index).unwrap();

        for id in 0..99 {
            let node = Node::new(id as TId, "127.0.0.1:8080".parse().unwrap());
            server.tables.update(&node);
        }

//        server.tables.buckets.iter().flat_map(|bucket| bucket.data.iter().collect()).collect().len();
//        writeln!(io::stdout(), "table size {}", server.tables.len()).unwrap();

        let count = server.tables.find(&82, 3);
        count.iter().for_each(|node|{
            writeln!(io::stdout(), "-->{}", node.id).unwrap();
        });

        let count = server.tables.pop_oldest();
        writeln!(io::stdout(), "table old node count {}", count.len());
        count.iter().for_each(|node|{writeln!(io::stdout(), "{}", node.id).unwrap();});
        writeln!(io::stdout(), "table size {}", server.tables.buckets().len()).unwrap();
    }
}