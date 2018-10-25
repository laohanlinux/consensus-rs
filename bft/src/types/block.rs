use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::encoding::msgpack::*;
use cryptocurrency_kit::ethkey::signature::*;
use cryptocurrency_kit::ethkey::{Address, Secret, Signature};
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use std::io::Cursor;

use super::transaction::Transaction;
use super::votes::Votes;
use super::{Bloom, Difficulty, Gas, Height, Timestamp};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub prev_hash: Hash,
    pub proposer: Address,
    pub root: Hash,
    pub tx_hash: Hash,
    pub receipt_hash: Hash,
    pub bloom: Bloom,
    pub difficulty: Difficulty,
    pub height: Height,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Vec<u8>>,
}

implement_cryptohash_traits! {Header}

impl Header {
    pub fn new(
        prev_hash: Hash,
        proposer: Address,
        root: Hash,
        tx_hash: Hash,
        receipt_hash: Hash,
        bloom: Bloom,
        diff: Difficulty,
        height: Height,
        gas_limit: Gas,
        gas_used: Gas,
        tm: Timestamp,
        extra: Option<Vec<u8>>,
    ) -> Self {
        Header {
            prev_hash,
            proposer,
            root,
            tx_hash,
            receipt_hash,
            bloom,
            difficulty: diff,
            height,
            gas_limit,
            gas_used,
            time: tm,
            extra,
        }
    }

    pub fn zero_header() -> Header {
        Header {
            prev_hash: Hash::zero(),
            proposer: Address::from(0),
            root: Hash::zero(),
            tx_hash: Hash::zero(),
            receipt_hash: Hash::zero(),
            bloom: 0,
            difficulty: 0,
            height: 0,
            gas_limit: 0,
            gas_used: 0,
            time: 0,
            extra: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    #[serde(rename = "tx")]
    transactions: Vec<Transaction>,
    votes: Option<Votes>, // the first vote is proposer's vote
}

impl Block {
    pub fn new(header: Header, txs: Vec<Transaction>, votes: Option<Votes>) -> Self {
        Block {
            header,
            transactions: txs,
            votes,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn votes(&self) -> Option<&Votes> {
        self.votes.as_ref()
    }

    pub fn mut_votes(&mut self) -> Option<&mut Votes> {
        self.votes.as_mut()
    }
}
