use actix::prelude::*;
use futures::{Future, Stream};
use tokio;
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
use codec::{RawCodec, RawMessage, P2PMessage, Request , RequestPayload, Response, ResponsePayload, TId, TAddr, TValue, TData};

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

pub struct  TcpServer {
    pub srv: Addr<Server>,
}

impl Actor for TcpServer {
    type Context = Context<Self>;
}

#[derive(Message)]
struct TcpConnect(pub TcpStream, pub TValue);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for TcpServer {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        writeln!(io::stdout(), "receive a client connect").unwrap();
        let addr = self.srv.clone();
        // For each incoming connection we create `Session` actor
        // with out server address.
        let peer_addr = msg.0.peer_addr().unwrap();
        let id = farmhash::hash64(peer_addr.to_string().as_bytes());
        let node = Node::new(id, peer_addr);
        session::Session::create(move |ctx: &mut Context<session::Session>|{
            let (r, w) = msg.0.split();
            // 注册反序列化
            session::Session::add_stream(FramedRead::new(r, RawCodec), ctx);
            // 注册序列化
            session::Session::new(node, addr, actix::io::FramedWrite::new(w, RawCodec, ctx))
        });
    }
}

#[derive(Message)]
struct TryConnect(pub TcpStream, pub TAddr);

impl Handler<TryConnect> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TryConnect, _:&mut Context<Self>) {
        writeln!(io::stdout(), "try to connect {}", msg.1.to_string()).unwrap();
        let id = farmhash::hash64(msg.1.to_string().as_bytes());
        let peer_addr = msg.1.to_string().parse().unwrap();
        let node = Node::new(id, peer_addr);
        let addr = self.srv.clone();
        let (r, w) = msg.0.split();
        session::Session::create(move |ctx|{
            // 注册反序列化
            session::Session::add_stream(FramedRead::new(r, RawCodec), ctx);
            // 注册序列化
            session::Session::new_peer(node, addr, actix::io::FramedWrite::new(w, RawCodec, ctx))
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
    node: Node<TId, TAddr>,
    kad_srv: service::Service<TId, TAddr, KNodeTable<TId, TAddr>, TData>,
    sessions: HashMap<TId, Addr<session::Session>>,
}

impl Default for Server{
    fn default() -> Server{
        let table = KNodeTable::new(0);
        let srv = service::Service::new(table);
        Server {
            node: Node::new(0, "127.0.0.1:12233".parse().unwrap()),
            kad_srv: srv,
            sessions: HashMap::new(),
        }
    }
}

impl Server {
    pub fn new(node: Node<TId, TAddr>) -> Server {
        let table = KNodeTable::new(node.id);
        let srv = service::Service::new_with_id(table, node.id);
        Server{
            node,
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
        writeln!(io::stdout(),"new connect is comming...").unwrap();
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
        writeln!(io::stdout(), "server receive session msg--> {:?}", msg.payload).unwrap();
        let mut handle = self.kad_srv.mut_handle();
        let request = msg.clone();
        let response = match msg {
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::Ping} => {
                handle.on_ping(&caller);
                // TODO
                KadResponse::new(request, self.node.clone(), ResponsePayload::NoResult)
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::FindNode(id)} => {
                let nodes = handle.on_find_node(&caller, &id);
                KadResponse::new(request, self.node.clone(), ResponsePayload::NodesFound(nodes))
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::FindValue(_)} => {
                unimplemented!()
            },
            KadRequest{caller: caller, request_id: rid, payload: RequestPayload::Store(_, _)} => {
                unimplemented!()
            },
        };
        response
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
    use std::thread;
    use std::time::Duration;

    use futures::Future;
    use rand;
    use rand::Rng;
    use actix::{self, msgs, Actor, Addr, Arbiter, Context, System};
    use farmhash;
    use tokio;
    use client::Client;

    fn start_server(addr: String) -> Addr<TcpServer> {
        let system = System::new("test-server");
        let id = farmhash::hash64(addr.as_bytes());
        let node = Node::new(id, addr.parse().unwrap());
        let mut srv = Server::new(node);
        let server:Addr<_> = srv.start();

        let addr = net::SocketAddr::from_str(&addr).unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        TcpServer::create(move |ctx| {
            ctx.add_message_stream(listener.incoming().map_err(|_|()).map(|st|{
                let addr = st.peer_addr().unwrap();
                TcpConnect(st, addr.to_string().as_bytes().to_vec())
            }));
            writeln!(io::stdout(), "Running chat server on {:?}", addr).unwrap();
            TcpServer{srv:server}
        })
    }


    fn start_client(srv_addr: String){

        //let system = System::new("test-client");
        System::run(move ||{
            writeln!(io::stdout(), "start client").unwrap();
            let addr = net::SocketAddr::from_str(&srv_addr).unwrap();
            tokio::spawn(TcpStream::connect(&addr)
                .and_then(|stream|{
                    let addr = Client::create(|ctx|{
                        let local_addr = stream.local_addr().unwrap();
                        let id = farmhash::hash64(local_addr.to_string().as_bytes());
                        let local_node = Node::new(id, local_addr);
                        let (r, w) = stream.split();
                        ctx.add_stream(FramedRead::new(r, OutboundCode));
                        Client{
                            node: local_node,
                            framed: actix::io::FramedWrite::new(
                                w,
                                OutboundCode,
                                ctx,
                            ),
                        }
                    });

                    ::futures::future::ok(())
                }).map_err(|e| {
                  println!("Can not connect to server: {}", e);
                  ::std::process::exit(1)
                }),
            );
        });
    }

    #[test]
    fn test_peer() {
        // tcp_addr1 --> connect ---> tcp_addr2
        // session:xxx   ----------> session:xxxx
        let tcp_addr1 = start_server("127.0.0.1:8888".to_string());
        let tcp_addr2 = start_server("127.0.0.1:8889".to_string());

        System::run(move||{
            let addr = net::SocketAddr::from_str("127.0.0.1:8889").unwrap();
            tokio::spawn(TcpStream::connect(&addr)
                .and_then(move |stream|{
                    // try to connect peer
                    tcp_addr1.do_send( TryConnect(stream,"127.0.0.1:8889".parse().unwrap()));
                    ::futures::future::ok(())
                })
                .map_err(|e|{()}));
        });
    }

    #[test]
    fn test_server(){
        let join_server = thread::spawn(move ||{
            start_server("127.0.0.1:8888".to_string());
        });

        let join_client = thread::spawn(move||{
            start_client("127.0.0.1:8888".to_string());
        });

        join_server.join().unwrap();
        join_client.join().unwrap();
    }

//    fn new_peers(n: isize) -> Vec<Server>{
//        let mut peers = Vec::new();
//        for i in 0..n {
//            let id =  rand::thread_rng().gen_range(1024, 4096);
////            let addr = format!("127.0.0.1:{}", id);
////            let id = farmhash::hash64(addr.as_bytes());
////            let node:Node<TId, TAddr> = Node::new(id, addr.parse().unwrap());
//            let server:Server = Server::new(id);
//            peers.push(server);
//        }
//        peers
//    }

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