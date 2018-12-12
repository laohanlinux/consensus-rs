use std::borrow::Cow;
use std::io::Cursor;
use std::str::FromStr;

use libp2p::{PeerId, Multiaddr};
use cryptocurrency_kit::crypto::{CryptoHash, Hash, hash};
use cryptocurrency_kit::storage::values::StorageValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Message)]
pub enum P2PMsgCode {
    Handshake,
    Transaction,
    Block,
    Consensus,
    Sync,
}

implement_storagevalue_traits! {P2PMsgCode}
implement_cryptohash_traits! {P2PMsgCode}

#[derive(Debug, Clone, Copy)]
pub enum BoundType {
    InBound,
    OutBound,
}

#[derive(Debug, Clone, Deserialize, Serialize, Message)]
pub struct RawMessage {
    header: Header,
    payload: Payload,
}

implement_storagevalue_traits! {RawMessage}
implement_cryptohash_traits! {RawMessage}

impl RawMessage {
    pub fn new(header: Header, payload: Vec<u8>) -> Self {
        RawMessage {
            header: header,
            payload: payload,
        }
    }

    pub(crate) fn header(&self) -> &Header {
        &self.header
    }

    pub(crate) fn mut_header(&mut self) -> &mut Header {
        &mut self.header
    }

    pub(crate) fn payload(&self) -> &Payload {
        &self.payload
    }

    pub(crate) fn mut_payload(&mut self) -> &mut Payload {
        &mut self.payload
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Header {
    pub code: P2PMsgCode,
    pub ttl: usize,
    pub create_time: u64,
}

implement_cryptohash_traits! {Header}
implement_storagevalue_traits! {Header}

impl Header {
    pub fn new(code: P2PMsgCode, ttl: usize, create_time: u64) -> Self {
        Header { code: code, ttl: ttl, create_time: create_time }
    }
}

pub type Payload = Vec<u8>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Handshake {
    version: String,
    peer_id: String,
    genesis: Hash,
}

implement_storagevalue_traits! {Handshake}
implement_cryptohash_traits! {Handshake}

impl Handshake {
    pub fn new(version: String, peer_id: PeerId, genesis: Hash) -> Self {
        let peer_id = peer_id.to_base58();
        Handshake {
            version: version,
            peer_id: peer_id,
            genesis: genesis,
        }
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId::from_str(&self.peer_id).unwrap()
    }

    pub fn genesis(&self) -> &Hash {
        &self.genesis
    }
}