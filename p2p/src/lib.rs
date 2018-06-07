extern crate farmhash;
extern crate dht;

use dht::*;
use std::net;

pub struct WhisperNodeTable {
    pub node: Option<u64>,
}

impl GenericNodeTable<u64, net::SocketAddr> for WhisperNodeTable {
    fn random_id(&self) ->WhisperNodeTable{
        WhisperNodeTable{
            node: None,
        }
    }
    fn update(&mut self, node: &Node<u64, net::SocketAddr>) -> bool {
        match self.node {
            Some(..) => false,
            None => {
                self.node = Some(node.clone());
                true
            }
        }
    }

    fn pop_oldest(){}
    fn find(){}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
