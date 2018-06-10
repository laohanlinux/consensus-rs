extern crate farmhash;
extern crate byteorder;
extern crate bytes;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate rand;
extern crate rustc_serialize;

#[macro_use]
extern crate log;


#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate actix;

use std::net;
use std::marker;

use kad::*;
use kad::protocol::*;

mod peer;
mod codec;
mod session;
mod server;
pub mod kad;

pub struct P2PProtocol{}

impl Protocol for P2PProtocol {
    type Id = u64;
    type Addr = net::SocketAddr;
    type Value = u64;
    /// Parse request from binary data.
    fn parse_request(&self, data: &[u8]) -> Request<Self::Id, Self::Addr, Self::Value> {
        let address = "127.0.0.1:800";
        let id = farmhash::hash64(address.as_bytes());
        let node = Node{address: address.parse().unwrap(), id: id};
        Request{
            caller: node,
            request_id: id,
            payload: RequestPayload::Ping,
        }
    }

    /// Format response to binary data.
    fn format_response(&self, resp: Response<Self::Id, Self::Addr, Self::Value>) -> Vec<u8> {
        vec![]
    }
}
fn ping_callback<TId, TAddr>(node: &Node<TId, TAddr>, flag: bool) {}

fn find_node<TId, TAddr>(node: Vec<Node<TId, TAddr>>) {}

fn find_value<TValue: Send + Sync +Clone, TId, TAddr>(value: Option<TValue>, nodes: Vec<Node<TId, TAddr>>) {

}

//impl GenericNodeTable<u64, net::SocketAddr> for WhisperNodeTable {
//    fn random_id(&self) ->WhisperNodeTable{
//        WhisperNodeTable{
//            node: None,
//        }
//    }
//
//    // 更新节点
//    fn update(&mut self, node: &Node<u64, net::SocketAddr>) -> bool {
//        match self.node {
//            Some(..) => false,
//            None => {
//                self.node = Some(node.clone());
//                true
//            }
//        }
//    }
//
//    fn find(&self, id: &u64, _count: usize) ->Vec<Node<u64, net::SocketAddr>>{
//        if let Some(ref node) = self.node {
//            if node.id == *id {
//                vec![node.clone()]
//            }else {
//                vec![]
//            }
//        }else {
//            vec![]
//        }
//    }
//
//    fn pop_oldest(&mut self) -> Vec<Node<u64, net::SocketAddr>>{
//        let result = self.node.or_else(||vec![]).unwrap();
//        self.node = Node;
//        result
//    }
//
//}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
