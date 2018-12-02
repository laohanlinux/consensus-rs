use std::borrow::Cow;
use std::io::Cursor;

use cryptocurrency_kit::crypto::{CryptoHash, Hash, hash};
use cryptocurrency_kit::storage::values::StorageValue;
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize, Message)]
pub struct RawMessage {
    header: Header,
    payload: Payload,
}

implement_storagevalue_traits! {RawMessage}
implement_cryptohash_traits! {RawMessage}

impl RawMessage {
    fn new(header: Header, payload: Vec<u8>) -> Self {
        RawMessage{
            header: header,
            payload: payload,
        }
    }

    fn header(&self) -> &Header {
        &self.header
    }

    fn mut_header(&mut self) -> &mut Header {
        &mut self.header
    }

    fn payload(&self) -> &Payload {
        &self.payload
    }

    fn mut_payload(&mut self) -> &mut Payload {
        &mut self.payload
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Header {
    ttl: usize,
    create_time: u64,
}

implement_cryptohash_traits! {Header}
implement_storagevalue_traits! {Header}

pub type Payload = Vec<u8>;