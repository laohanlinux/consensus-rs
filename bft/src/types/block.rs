use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash, EMPTY_HASH};
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::encoding::msgpack::*;
use cryptocurrency_kit::ethkey::signature::*;
use cryptocurrency_kit::ethkey::{Address, Secret, Signature};
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use std::io::Cursor;
use std::borrow::Cow;

use super::transaction::Transaction;
use super::votes::Votes;
use super::{Bloom, Difficulty, Gas, Height, Timestamp};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub prev_hash: Hash,
    pub proposer: Address,
    pub root: Hash,
    // state root
    pub tx_hash: Hash,
    // transactions root
    pub receipt_hash: Hash,
    // receipt_root
    pub bloom: Bloom,
    pub difficulty: Difficulty,
    pub height: Height,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    pub time: Timestamp,
    #[serde(default)]
    pub extra: Option<Vec<u8>>,
    #[serde(default)]
    pub votes: Option<Votes>,
    #[serde(skip_serializing, skip_deserializing)]
    hash_cache: Option<Hash>,
}

implement_cryptohash_traits! {Header}
implement_storagevalue_traits! {Header}

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
        votes: Option<Votes>,
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
            votes,
            hash_cache: None,
        }
    }

    pub fn block_hash(&self) -> Hash {
        let mut header = self.clone();
        header.votes = None;
        <Header as CryptoHash>::hash(&header)
    }

    pub fn new_mock(pre_hash: Hash, proposer: Address, tx_hash: Hash, height: Height, tm: Timestamp, extra: Option<Vec<u8>>) -> Self {
        Self::new(pre_hash, proposer, EMPTY_HASH, tx_hash, EMPTY_HASH, 0, 0, height, 0, 0, tm, None, extra)
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
            votes: None,
            hash_cache: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HeaderBytes<'a> {
    pub prev_hash: Cow<'a, Hash>,
    pub proposer: Address,
    pub root: Cow<'a, Hash>,
    // state root
    pub tx_hash: Cow<'a, Hash>,
    // transactions root
    pub receipt_hash: Cow<'a, Hash>,
    // receipt_root
    pub bloom: Bloom,
    pub difficulty: Difficulty,
    pub height: Height,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    pub time: Timestamp,
    #[serde(default)]
    pub extra: Option<Cow<'a, Vec<u8>>>,
}

//impl HeaderBytes<'_> {
//    fn new(header: &Header) -> Self {
//        HeaderBytes {
//            prev_hash: Cow::from(&header.prev_hash),
//            proposer: header.proposer.clone(),
//            root: Cow::from(header.root),
//            tx_hash: Cow::from(header.tx_hash),
//            receipt_hash: Cow::from(header.receipt_hash),
//            bloom: header.bloom,
//            difficulty: header.difficulty,
//            height: header.height,
//            gas_limit: header.gas_limit,
//            gas_used: header.gas_used,
//            time: header.time,
//            extra: None,
//        }
//    }
//}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header,
    transactions: Vec<Transaction>,
}

implement_cryptohash_traits! {Block}
implement_storagevalue_traits! {Block}

pub type Blocks = Vec<Block>;

impl Block {
    pub fn new(header: Header, txs: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions: txs,
        }
    }

    pub fn new2(header: Header, transactions: Vec<Transaction>) -> Self {
        Block {
            header: header,
            transactions: transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        self.header.block_hash()
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn mut_header(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn height(&self) -> Height { self.header.height }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn mut_transactions(&mut self) -> &mut Vec<Transaction> {
        &mut self.transactions
    }

    pub fn coinbase(&self) -> Address {
        let coinbase = self.header.proposer;
        coinbase
    }

    pub fn add_votes(&mut self, signatures: Vec<Signature>) {
        let ref mut header = self.header;
        let votes = header.votes.get_or_insert(Votes::new(vec![]));
        votes.add_votes(&signatures);
    }

    pub fn votes(&self) -> Option<&Votes> {
        self.header.votes.as_ref()
    }

    pub fn mut_votes(&mut self) -> Option<&mut Votes> {
        self.header.votes.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn header() {
        let header = Header::zero_header();
        writeln!(io::stdout(), "{:#?}", header).unwrap();
        let j_str = serde_json::to_string(&header).unwrap();
        writeln!(io::stdout(), "{}", j_str).unwrap();
    }
}