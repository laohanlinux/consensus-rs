use actix::prelude::*;
use futures::Stream;
use tokio_io::codec::FramedRead;
use tokio_io::AsyncRead;
use tokio_tcp::{TcpListener, TcpStream};
use farmhash;

use std::collections::{HashMap};
use std::net;
use std::io::{self, Write};

use session;
use kad::service;
use kad::knodetable::*;
use kad::base::Node;
use codec::{Codec, Request , RequestPayload, Response, ResponsePayload, TId, TAddr, TValue, TData};

type KadRequest = Request<TId, TAddr, TValue>;
type KadRequestPayload = RequestPayload<TId, TValue>;
type KadResponse = Response<TId, TAddr, TValue>;
type KadResponsePayload = ResponsePayload<TId, TAddr, TValue>;

/// Message for server communications

/// New chat session is created
pub struct Connect{
    pub node: Node<TId, TAddr>,
    pub addr: Addr<session::Session>,
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
pub struct InnerMessage{
    /// Id of the client session
    pub id: TId,
    /// Peer message
    pub msg: String,
}

impl InnerMessage {
    pub fn new(id: TId, msg: String) ->InnerMessage {
        InnerMessage{
            id,
            msg,
        }
    }
}

#[derive(Message)]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for Server {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        // For each incoming connection we create `Session` actor
        // with out server address.
        let id = farmhash::hash64(msg.1.to_string().as_bytes());
        session::Session::create(move ||{
            let (r, w) = msg.0.split();
            session::Session::add_stream(FramedRead::new(r, Codec), ctx);
            session::Session::new(id, self.clone(), actix::io::FramedWrite::new(w, ChatCodec, ctx))
        });
    }
}

/// Mock message just for test
#[derive(Debug)]
pub struct MockMessage{
    pub msg: String,
}

impl Message for MockMessage {
    type Result = String;
}

pub struct Server {
    kad_srv: service::Service<TId, TAddr, KNodeTable<TId, TAddr>, TData>,
    sessions: HashMap<TId, Addr<session::Session>>,
}

impl Default for Server{
    fn default() -> Server{
        let table = KNodeTable::new(0);
        let srv = service::Service::new(table);
        Server {
            kad_srv: srv,
            sessions: HashMap::new(),
        }
    }
}

impl Server {
    pub fn new(node_id: TId) -> Server {
        let table = KNodeTable::new(node_id);
        let srv = service::Service::new_with_id(table, node_id);
        Server{
            kad_srv:srv,
            sessions: HashMap::new(),
        }
    }

    /// Send message to all nodes
    pub fn send_message(&self, msg: InnerMessage) {
        match msg {
            InnerMessage{id:0, msg: msg} => {
                self.sessions.iter().for_each(|(_, addr)|{
                    addr.do_send(session::Message(msg.clone()));
                });
            },
            InnerMessage{id: id, msg: msg} => {
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

impl Handler<MockMessage> for Server {
    type Result = String;

    fn handle(&mut self, msg: MockMessage, _: &mut Context<Self>) -> String{
        writeln!(io::stdout(), "MockMessage handle").unwrap();
        msg.msg
    }
}

#[cfg(test)]
mod test {
    use codec::*;
    use kad::base::Node;
    use kad::base::GenericAPI;
    use kad::base::GenericNodeTable;
    use std::io::{self, Write};
    use super::*;
    use std::net;
    use std::str::FromStr;

    use rand;
    use rand::Rng;
    use actix::{self, msgs, Actor, Addr, Arbiter, Context, System};
    use farmhash;
    use tokio;

    #[test]
    fn test_server(){
        let system = System::new("test");
        let addr = "127.0.0.1:8888";
        let srv = new_server(addr.as_bytes());
        let server:Addr<_> = srv.start();

        let peer:Addr<Server> = {
          new_server("127.0.0.1:8881".as_bytes()).start()
        };

        System::run(move||{
            // Start p2p server
            let addr = net::SocketAddr::from_str(addr);
            let listener = TcpListener::bind(&addr).unwrap();
            // send a message to server
            let res = server.send(MockMessage{msg: "hello".to_string()});
            writeln!(io::stdout(), "Coming...").unwrap();
            tokio::spawn(res.map(|res|{
                writeln!(io::stdout(), "server return value {:?}", res).unwrap();
                System::current().stop();
            }).map_err(|_|()));


        });

    }

    fn new_server(id: &[u8]) -> Server{
        let id = farmhash::hash64(id);
        let mut server:Server = Server::new(id);
        server
        // insert a node
//        for id in 1..1000 {
//            let addr  = format!("127.0.0.1:{}", id);
//            let id = farmhash::hash64(addr.as_bytes());
//            let node:Node<TId, TAddr> = Node::new(id, addr.parse().unwrap());
//            let handle = server.kad_srv.mut_handle();
//            handle.on_ping(&node);
//        }
    }

    fn new_boot_node(id: &[u8]) -> Server {
        new_server(id)
    }

    fn new_peers(n: isize) -> Vec<Server>{
        let mut peers = Vec::new();
        for i in 0..n {
            let id =  rand::thread_rng().gen_range(1024, 4096);
//            let addr = format!("127.0.0.1:{}", id);
//            let id = farmhash::hash64(addr.as_bytes());
//            let node:Node<TId, TAddr> = Node::new(id, addr.parse().unwrap());
            let server:Server = Server::new(id);
            peers.push(server);
        }
        peers
    }

    fn contract_peers(boot_node:&mut Server, peers: &mut Vec<Server>) {
        for server in peers.iter() {
        }
    }

    #[test]
    fn test_connect(){
    }

    //#[test]
//    fn test_server(){
//        let id = 100;
//        let mut server = super::Server::new(id);
//        let index = server.tables.random_id();
//        writeln!(io::stdout(), "random id {}", index).unwrap();
//
//        // insert a node
//        for id in 1..1000 {
//            let addr  = format!("127.0.0.1:{}", id);
//            let id = farmhash::hash64(addr.as_bytes());
//            writeln!(io::stdout(), "new node id {}", id).unwrap();
//            let node = Node::new(id , addr.parse().unwrap());
//            let _ = server.tables.update(&node);
//            //assert_eq!(existFlag, false, "node id: {}", id);
//        }
//
//        let count = server.tables.buckets().iter().fold(0, |acc, bucket|{acc+bucket.size()});
//        writeln!(io::stdout(), "table size {}", count);
//
//        // find a node
//        let count = server.tables.find(&82, 3);
//        count.iter().for_each(|node|{
//            writeln!(io::stdout(), "-->{}", node.id).unwrap();
//        });
//
//        // pop a node
//        let count = server.tables.pop_oldest();
//        writeln!(io::stdout(), "table old node count {}", count.len());
//        count.iter().for_each(|node|{writeln!(io::stdout(), "{}", node.id).unwrap();});
//        writeln!(io::stdout(), "table size {}", server.tables.buckets().len()).unwrap();
//    }
}