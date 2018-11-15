mod core;
mod round_state;

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
use std::collections::HashMap;
use std::sync::RwLock;

use consensus::types::View;
use consensus::validator::{self, ImplValidatorSet, ValidatorSet, fn_selector};

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
    pub code: MessageType,
    pub msg:    Vec<u8>,
    pub address: Address,
    pub signature: Option<Signature>,
    pub commit_seal: Option<Signature>,
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

struct MessageManage<V=ImplValidatorSet> where V: ValidatorSet {
    view: View,
    val_set: V,
    messages: HashMap<Address, Message>,
}

impl<V> std::fmt::Debug for MessageManage <V>
    where V: ValidatorSet,
{
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         write!(f, "(view: {:?})", self.view)
     }
}

impl<V> MessageManage <V>
    where V: ValidatorSet,
{
    pub fn new(view: View, val_set: V) -> Self {
        MessageManage{
            view: view,
            val_set: val_set,
            messages: HashMap::new(),
        }
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn add(&mut self, msg: Message) -> Result<(), String> {
        self.val_set.get_by_address(msg.address).ok_or("".to_string())?;
        self.add_verify_message(msg)
    }

    pub fn values(&self) -> Vec<Message> {
        let mut v:Vec<Message> = vec![];
        self.messages.values().cloned().for_each(|item| {
            v.push(item);
        });
        v
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn get_message(&self, address: Address) -> Option<&Message> {
       self.messages.get(&address)
    }

    fn verify(&self, msg: &Message) -> Result<(), String> {
        if self.val_set.get_by_address(msg.address).is_none(){
            return Err("".to_string());
        }
        Ok(())
    }

    fn add_verify_message(&mut self, msg: Message) -> Result<(), String> {
        self.messages.insert(msg.address, msg);
        Ok(())
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

    #[test]
    fn message_manager(){
        let mut msg = MessageManage::new(View{round: 1, height: 1}, new_zero_validator_set());

        assert_eq!(msg.len(), 0);
        assert_eq!(msg.view(), View{round:1, height:1});
        assert_eq!(msg.values().len(), 0);

        // add a message
        {
            assert_eq!(msg.add(Message{code: MessageType::AcceptRequest, msg: vec![1, 3 ,4], address: 100.into(), signature: None, commit_seal: None}).is_ok(), true);
            assert_eq!(msg.add(Message{code: MessageType::AcceptRequest, msg: vec![1, 3 ,4], address: 101.into(), signature: None, commit_seal: None}).is_ok(), false);
        }

        // get a message
        {
            assert_eq!(msg.get_message(100.into()).unwrap().code, MessageType::AcceptRequest);
        }
        assert_eq!(msg.len(), 1);
        assert_eq!(msg.values().len(), 1);
    }

    #[test]
    fn current_message_manager(){
        use std::thread;
        use std::sync::RwLock;
        use std::sync::Arc;
        let mut msg_manager = Arc::new(RwLock::new(MessageManage::new(View{round:1, height:1}, new_zero_validator_set())));

        let mut joins:Vec<thread::JoinHandle<i32>> = vec![];
        (0..100).for_each(|_|{
            let arc_msg_manager = msg_manager.clone();
            let join = thread::spawn(move ||{
                arc_msg_manager.write().unwrap().add(Message{code: MessageType::AcceptRequest, msg: vec![1, 3 ,4], address: 100.into(), signature: None, commit_seal: None});
                1
            });
        });

        for join in joins {
            assert_eq!(join.join().unwrap(), 1);
        }

        assert_eq!(msg_manager.write().unwrap().len(), 1);
    }

    fn new_zero_validator_set() -> ImplValidatorSet {
        let mut address_list = vec![
            Address::from(100),
            Address::from(10),
            Address::from(21),
            Address::from(31),
            Address::from(3),
        ];
        ImplValidatorSet::new(&address_list, Box::new(fn_selector))
    }
}
