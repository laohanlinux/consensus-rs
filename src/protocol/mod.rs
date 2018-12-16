use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::ethkey::{public_to_address, recover_bytes};
use cryptocurrency_kit::ethkey::{
    Address, KeyPair, Message as sMessage, Public, Secret, Signature,
};
use cryptocurrency_kit::storage::values::StorageValue;
use chrono::Local;
use serde::{Deserialize, Serialize};
use ::actix::prelude::*;

use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash as stdHash, Hasher};
use std::io::Cursor;
use std::sync::RwLock;

use crate::{
    consensus::types::View,
    consensus::validator::{self, fn_selector, ImplValidatorSet, ValidatorSet},
    types::EMPTY_ADDRESS,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum State {
    AcceptRequest = 1,
    PrePrepared,
    Prepared,
    Committed,
}

implement_cryptohash_traits! {MessageType}
implement_storagevalue_traits! {MessageType}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
pub enum MessageType {
    Preprepare = 1,
    Prepare,
    Commit,
    RoundChange,
}

#[derive(Debug, Clone, Message, Deserialize, Serialize)]
pub struct GossipMessage {
    pub code: MessageType,
    pub create_time: u64,
    pub msg: Vec<u8>,
    #[serde(default)]
    pub signature: Option<Signature>,
    #[serde(default)]
    pub commit_seal: Option<Signature>,
    #[serde(skip_serializing, skip_deserializing)]
    pub address: Address,
}

implement_cryptohash_traits! {GossipMessage}
implement_storagevalue_traits! {GossipMessage}


impl stdHash for GossipMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = CryptoHash::hash(self);
        hash.as_ref().hash(state);
    }
}

impl PartialEq for GossipMessage {
    fn eq(&self, other: &GossipMessage) -> bool {
        let (hash1, hash2) = (CryptoHash::hash(self), CryptoHash::hash(other));
        hash1 == hash2
    }
}

impl Eq for GossipMessage {}

impl GossipMessage {
    pub fn new(code: MessageType, msg: Vec<u8>, commit_seal: Option<Signature>) -> Self {
        GossipMessage {
            code,
            create_time: 0,
            msg,
            signature: None,
            commit_seal,
            address: *EMPTY_ADDRESS,
        }
    }

    pub fn set_sign(&mut self, secret: &Secret) {
        let hash = self.sign_digest();
        let signature = hash.sign(secret).unwrap();
        self.signature = Some(signature);
    }

    pub fn set_seal(&mut self, digest: Hash, secret: &Secret) {
        let seal_sign = digest.sign(secret).unwrap();
        self.commit_seal = Some(seal_sign);
    }

    pub fn get_sign(&self) -> Option<&Signature> {
        return self.signature.as_ref();
    }

    pub fn address(&mut self) -> Result<Address, String> {
        let result = match self.signature {
            Some(ref signature) => {
                let bytes = self.sign_payload();
                recover_bytes(signature, &bytes)
                    .map(|ref public_key| public_to_address(public_key))
                    .map_err(|_| "failed to recover public key from signature".to_string())
            }
            None => Err("invalid signature".to_string()),
        };

        self.address = result?;
        Ok(self.address)
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.into_bytes()
    }

    pub(crate) fn msg(&self) -> &Vec<u8> {
        &self.msg
    }

    pub(crate) fn sign_digest(&self) -> Hash {
        let bytes = self.sign_payload();
        CryptoHash::hash(&bytes)
    }

    pub(crate) fn sign_payload(&self) -> Vec<u8> {
        let mut msg = self.clone();
        msg.signature = None;
        msg.into_bytes()
    }

    pub(crate) fn trace(&self) -> String {
        format!("code:{:?}, address:{:?}", self.code, self.address)
    }
}

pub struct MessageManage<RHS = ImplValidatorSet>
    where
        RHS: ValidatorSet,
{
    view: View,
    val_set: RHS,
    messages: HashMap<Address, GossipMessage>,
}

impl<V> std::fmt::Debug for MessageManage<V>
    where
        V: ValidatorSet,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(view: {:?})", self.view)
    }
}

impl<V> MessageManage<V>
    where
        V: ValidatorSet,
{
    pub fn new(view: View, val_set: V) -> Self {
        MessageManage {
            view: view,
            val_set: val_set,
            messages: HashMap::new(),
        }
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn add(&mut self, msg: GossipMessage) -> Result<(), String> {
        self.val_set
            .get_by_address(msg.address)
            .ok_or_else(|| "".to_string())?;
        self.add_verify_message(msg)
    }

    pub fn values(&self) -> Vec<GossipMessage> {
        let mut v: Vec<GossipMessage> = vec![];
        self.messages.values().cloned().for_each(|item| {
            v.push(item);
        });
        v
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn get_message(&self, address: Address) -> Option<&GossipMessage> {
        self.messages.get(&address)
    }

    fn verify(&self, msg: &GossipMessage) -> Result<(), String> {
        if self.val_set.get_by_address(msg.address).is_none() {
            return Err("".to_string());
        }
        Ok(())
    }

    fn add_verify_message(&mut self, msg: GossipMessage) -> Result<(), String> {
        self.messages.insert(msg.address, msg);
        Ok(())
    }
}

pub(crate) fn to_priority(msg_code: MessageType, view: View) -> i64 {
    let priority = if msg_code == MessageType::RoundChange {
        (view.height * 1000) as i64
    } else {
        (view.height * 1000 + view.round + 10 + msg_code as u64) as i64
    };
    -priority
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptocurrency_kit::ethkey::Generator;
    use cryptocurrency_kit::ethkey::Random;
    use crate::protocol::{MessageType, GossipMessage};
    use std::io::{self, Write};
    use std::str::FromStr;

    fn new_message() -> (GossipMessage, KeyPair) {
        let key_pair = Random.generate().unwrap();
        let (round, height) = (rand::random::<u64>(), rand::random::<u64>());
        let mut v: Vec<u8> = Vec::with_capacity(16);
        v.write_fmt(format_args!("{}{}", round, height)).unwrap();
        let mut message = GossipMessage::new(MessageType::Prepare, v, None);
        message.set_sign(key_pair.secret());

        (message, key_pair)
    }

    #[test]
    fn eq() {
        assert_eq!(MessageType::Prepare, MessageType::Prepare);
        let a = MessageType::Prepare;
        let b = MessageType::Prepare;
        assert_eq!(a, b);
        assert!(MessageType::Prepare < MessageType::Commit);
        assert!(MessageType::Prepare <= MessageType::Commit);
    }

    #[test]
    fn message_serde() {
        let (mut message, _) = new_message();
        //        writeln!(io::stdout(), "{:?}", message.msg).unwrap();
        let json = serde_json::to_string_pretty(&message).unwrap();
        writeln!(io::stdout(), "{}", json).unwrap();
        let message_de: GossipMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(message.code, message_de.code);
        assert_eq!(message.msg, message_de.msg);
        assert_eq!(message.signature, message_de.signature);
        assert_eq!(message.address, message_de.address);
        assert_eq!(message.commit_seal, message_de.commit_seal);
    }

    #[test]
    fn message_sign() {
        use cryptocurrency_kit::ethkey::{Secret, KeyPair, public_to_address};
        use cryptocurrency_kit::ethkey::{recover_bytes, sign_bytes};

        let keypair = KeyPair::from_secret(
            Secret::from_str("7f3b0a324e13e5358c3fd686737acd7adf2e5556084ec6d9e48b497082b7ef98").unwrap())
            .unwrap();
        let mut msg = GossipMessage::new(MessageType::RoundChange, vec![1], None);
        msg.set_sign(keypair.secret());

        let payload = msg.sign_payload();

        let signature = sign_bytes(keypair.secret(), &payload).unwrap();
        let pubkey = recover_bytes(&signature, &payload).unwrap();
        println!("{:?}", pubkey);
        println!("{:?}", keypair.public());
        println!("{:?}", public_to_address(&pubkey));

        println!("{:?}", msg.address().unwrap());
        println!("{:?}", msg.address);
    }

//    #[test]
//    fn message_manager() {
//        let mut msg = MessageManage::new(
//            View {
//                round: 1,
//                height: 1,
//            },
//         new_zero_validator_set(),
//        );
//
//        assert_eq!(msg.len(), 0);
//        assert_eq!(
//            msg.view(),
//            View {
//                round: 1,
//                height: 1,
//            }
//        );
//        assert_eq!(msg.values().len(), 0);
//
//        // add a message
//        {
//            assert_eq!(
//                msg.add(GossipMessage {
//                    code: MessageType::AcceptRequest,
//                    msg: vec![1, 3, 4],
//                    address: 100.into(),
//                    signature: None,
//                    commit_seal: None,
//                })
//                    .is_ok(),
//                true
//            );
//            assert_eq!(
//                msg.add(GossipMessage {
//                    code: MessageType::AcceptRequest,
//                    msg: vec![1, 3, 4],
//                    address: 101.into(),
//                    signature: None,
//                    commit_seal: None,
//                })
//                    .is_ok(),
//                false
//            );
//        }
//
//        // get a message
//        {
//            assert_eq!(
//                msg.get_message(100.into()).unwrap().code,
//                MessageType::AcceptRequest
//            );
//        }
//        assert_eq!(msg.len(), 1);
//        assert_eq!(msg.values().len(), 1);
//    }
//
//    #[test]
//    fn current_message_manager() {
//        use std::sync::Arc;
//        use std::sync::RwLock;
//        use std::thread;
//        let mut msg_manager = Arc::new(RwLock::new(MessageManage::new(
//            View {
//                round: 1,
//                height: 1,
//            },
//            new_zero_validator_set(),
//        )));
//
//        let mut joins: Vec<thread::JoinHandle<i32>> = vec![];
//        (0..100).for_each(|_| {
//            let arc_msg_manager = msg_manager.clone();
//            let join = thread::spawn(move || {
//                arc_msg_manager.write().unwrap().add(GossipMessage {
//                    code: MessageType::Prepare,
//                    msg: vec![1, 3, 4],
//                    address: 100.into(),
//                    signature: None,
//                    commit_seal: None,
//                });
//                1
//            });
//        });
//
//        for join in joins {
//            assert_eq!(join.join().unwrap(), 1);
//        }
//
//        assert_eq!(msg_manager.write().unwrap().len(), 1);
//    }
//
//    fn new_zero_validator_set() -> ImplValidatorSet {
//        let mut address_list = vec![
//            Address::from(100),
//            Address::from(10),
//            Address::from(21),
//            Address::from(31),
//            Address::from(3),
//        ];
//        ImplValidatorSet::new(&address_list, Box::new(fn_selector))
//    }
//
//    fn new_zero_validator_set1() -> Box<ValidatorSet>
//    {
//        let mut address_list = vec![
//            Address::from(100),
//            Address::from(10),
//            Address::from(21),
//            Address::from(31),
//            Address::from(3),
//        ];
//        Box::new(ImplValidatorSet::new(&address_list, Box::new(fn_selector)))
//    }
//
//    #[test]
//    fn priority_queue() {
//        use chrono::Local;
//        use chrono_humanize::HumanTime;
//        use priority_queue::PriorityQueue;
//
//        // push dump message
//        {
//            let mut qp = PriorityQueue::new();
//            let (mut message, _) = new_message();
//            assert!(qp.push(message.clone(), 1).is_none());
//            assert!(qp.push(message.clone(), 1).is_some());
//        }
//
//        {
//            let mut qp = PriorityQueue::new();
//            (0..10).for_each(|idx: u8| {
//                let (mut message, _) = new_message();
//                message.msg.push(idx);
//                qp.push(message, idx);
//            });
//
//            (0..10).for_each(|_| {
//                let (message, idx) = qp.pop().unwrap();
//                writeln!(io::stdout(), "idx: {}, {:#?}", idx, message).unwrap();
//            });
//        }
//    }
}
