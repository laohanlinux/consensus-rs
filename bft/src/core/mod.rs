mod core;

use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::Signature;
use cryptocurrency_kit::ethkey::{Secret, KeyPair};
use cryptocurrency_kit::ethkey::{Message as sMessage};
use cryptocurrency_kit::crypto::{Hash, hash, CryptoHash};
use cryptocurrency_kit::storage::values::StorageValue;
// enc,dec
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use std::borrow::Borrow;
use std::borrow::Cow;
use std::io::Cursor;

implement_cryptohash_traits! {MessageType}
implement_storagevalue_traits! {MessageType}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
pub enum MessageType {
    AcceptRequest = 1,
    Preprepared,
    Prepared,
    Committed,
}

implement_cryptohash_traits! {Message}
implement_storagevalue_traits! {Message}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    code: MessageType,
    msg:    Vec<u8>,
    address: Address,
    signature: Option<Signature>,
    commit_seal: Option<Signature>,
}

impl Message {
    pub fn set_sign(&mut self, secret: &Secret) {
        let hash = self.sign_digest();
        let signature = hash.sign(secret).unwrap();
        self.signature = Some(signature);
    }

    pub fn get_sign(&self) -> Option<&Signature> {
        return self.signature.as_ref()
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.into_bytes()
    }

    pub(crate) fn sign_digest(&self) -> Hash {
        let bytes = self.sign_payload();
        bytes.hash()
    }

    pub(crate) fn sign_payload(&self) -> Vec<u8> {
        let mut msg = self.clone();
        msg.signature = None;
        msg.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq(){
        assert_eq!(MessageType::AcceptRequest, MessageType::AcceptRequest);
        let a = MessageType::AcceptRequest;
        let b = MessageType::AcceptRequest;
        assert_eq!(a, b);
        assert!(MessageType::AcceptRequest < MessageType::Preprepared);
        assert!(MessageType::AcceptRequest <= MessageType::Preprepared);
    }
}