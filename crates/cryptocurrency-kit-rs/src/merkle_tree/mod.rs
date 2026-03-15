use std::iter;


use crate::common::to_keccak;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub root: Option<Box<MerkleNode>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MerkleNode {
    pub data: Box<Vec<u8>>,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

impl MerkleTree {
    // just build the root
    pub fn new_merkle_tree(mut data: Vec<Vec<u8>>) -> MerkleTree {
        let clone_data = {
            if data.len() % 2 != 0 {
                let clone_data = {
                    data.last().unwrap()
                };
                Some(clone_data.clone())
            } else {
                None
            }
        };
        if clone_data.is_some() {
            data.push(clone_data.unwrap());
        }

        let mut nodes = vec![];

        data.iter().for_each(
            |dataum| nodes.push(MerkleNode::new(dataum)),
        );

        loop {
            let mut new_level = vec![];
            let (mut i, mut j) = (0, 0);
            while i < &nodes.len() / 2 {
                let node = MerkleNode::new_merkle_node(nodes[j].clone(), nodes[j + 1].clone());
                new_level.push(node);
                j += 2;
                i += 1;
            }
            nodes = new_level;
            if nodes.len() == 1 {
                break;
            }
        }
        MerkleTree { root: Some(Box::new(nodes.pop().unwrap())) }
    }
}

impl MerkleNode {
    fn new(data: &[u8]) -> MerkleNode {
        let mut mn: MerkleNode = Default::default();
        let keccak: [u8; 32] = to_keccak(data).into();
        mn.data = Box::new(keccak.to_vec());
        mn
    }
    fn new_merkle_node(left: MerkleNode, right: MerkleNode) -> MerkleNode {
        let mut merkle_tree_node: MerkleNode = Default::default();
        let mut hash_data = Vec::with_capacity(left.data.len() + right.data.len());
        hash_data.extend(iter::repeat(0).take(left.data.len() + right.data.len()));
        hash_data[..left.data.len()].clone_from_slice(&left.data);
        hash_data[left.data.len()..].clone_from_slice(&right.data);

        let hash: [u8; 32] = to_keccak(&hash_data).into();
        merkle_tree_node.data = Box::new(hash.to_vec());
        merkle_tree_node
    }
}

impl Default for MerkleNode {
    fn default() -> MerkleNode {
        MerkleNode {
            data: Box::new(vec![]),
            left: None,
            right: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn merkle_tree() {
        let vv = vec![vec![1, 3, 4], vec![4, 51, 3], vec![98]];
        let merkle_tree = MerkleTree::new_merkle_tree(vv);
        writeln!(io::stdout(), "root {:?}", merkle_tree.root.unwrap()).unwrap();
    }
}