use actix::prelude::*;

use std::net;
use super::base::{Node};

use std::collections::{HashMap};

pub struct Connect {
    pub addr: Node<u64, net::SocketAddr>,
}
impl actix::Message for Connect {
    type Result = usize;
}

#[derive(Message)]
pub struct Disconnect{
    pub id: usize,
}


/// Send message to specific room
#[derive(Message)]
pub struct Message {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

pub struct P2PServer {
    sessions: HashMap<u64, Node<u64, net::SocketAddr>>,
}

impl Default for P2PServer {
    fn default() -> P2PServer {
        P2PServer {
            sessions: HashMap::new(),
        }
    }
}

impl P2PServer {

    fn broadcast_message(&self, message: &str) {

    }

    fn send_message(&self, id: u64) {}
}