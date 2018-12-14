use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::ethkey::signature::*;
use cryptocurrency_kit::ethkey::{Address, Secret, Signature};
use cryptocurrency_kit::storage::keys::StorageKey;
use cryptocurrency_kit::storage::values::StorageValue;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use std::borrow::Cow;
use std::io::Cursor;

use crate::common::merkle_tree_root;
use super::Gas;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    #[serde(rename = "nonce")]
    account_nonce: u64,
    #[serde(rename = "price")]
    gas_price: u64,
    gas_limit: Gas,
    recipient: Option<Address>,
    amount: u64,
    #[serde(default)]
    payload: Vec<u8>,
    #[serde(rename = "sign")]
    signature: Option<Signature>,
    #[serde(skip_serializing, skip_deserializing)]
    hash: Option<Hash>,
}

impl CryptoHash for Transaction {
    fn hash(&self) -> Hash {
        hash(self.hash_payload())
    }
}

implement_storagevalue_traits! {Transaction}

impl Transaction {
    pub fn new(
        nonce: u64,
        to: Address,
        amount: u64,
        gas_limit: Gas,
        gas_price: u64,
        payload: Vec<u8>,
    ) -> Self {
        Transaction {
            account_nonce: nonce,
            gas_price: gas_price,
            gas_limit: gas_limit,
            recipient: Some(to),
            amount: amount,
            payload: payload,
            signature: None,
            hash: None,
        }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn gas(&self) -> Gas {
        self.gas_limit
    }
    pub fn gas_price(&self) -> Gas {
        self.gas_price
    }
    pub fn amount(&self) -> u64 {
        self.amount
    }
    pub fn nonce(&self) -> u64 {
        self.account_nonce
    }
    pub fn to(&self) -> Option<&Address> {
        self.recipient.as_ref()
    }
    pub fn get_hash(&self) -> Option<&Hash> {
        self.hash.as_ref()
    }
    pub fn pretty_json(&self) -> String {
        to_string(self).unwrap()
    }

    /// TODO
    pub fn sign(&mut self, _chain_id: u64, secret: &Secret) {
        let signature = sign_bytes(secret, &TransactionSignature::packet_signature(&self));
        self.signature = Some(signature.unwrap());
    }

    pub fn verify_sign(&self, _chai_id: u64) -> bool {
        if self.signature.is_none() {
            return false;
        }
        let payload = self.signature_payload();
        recover_bytes(self.signature.as_ref().unwrap(), &payload).is_ok()
    }

    pub fn set_hash(&mut self, hash: Hash) {
        self.hash = Some(hash)
    }

    pub fn set_signature(&mut self, sign: &Signature) {
        self.signature = Some(sign.clone());
    }

    pub fn signature_payload(&self) -> Vec<u8> {
        TransactionSignature::packet_signature(&self)
    }

    pub fn hash_payload(&self) -> Vec<u8> {
        TransactionSignature::packet_hash(&self)
    }
}

impl Eq for Transaction {}

impl PartialEq for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.get_hash().unwrap() == other.get_hash().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TransactionSignature {
    #[serde(rename = "nonce")]
    account_nonce: u64,
    #[serde(rename = "price")]
    gas_price: u64,
    gas_limit: Gas,
    recipient: Address,
    amount: u64,
    #[serde(default)]
    payload: Vec<u8>,
    #[serde(rename = "sign")]
    signature: Option<Signature>,
}

implement_storagevalue_traits! {TransactionSignature}
implement_cryptohash_traits! {TransactionSignature}

impl TransactionSignature {
    fn packet_hash(tx: &Transaction) -> Vec<u8> {
        let sign = tx.signature.as_ref().unwrap().clone();
        let signature = TransactionSignature {
            account_nonce: tx.account_nonce,
            gas_price: tx.gas_price,
            gas_limit: tx.gas_limit,
            recipient: tx.recipient.unwrap(),
            amount: tx.amount,
            payload: tx.payload.clone(),
            signature: Some(sign),
        };
        signature.into_bytes()
    }

    fn packet_signature(tx: &Transaction) -> Vec<u8> {
        let signature = TransactionSignature {
            account_nonce: tx.account_nonce,
            gas_price: tx.gas_price,
            gas_limit: tx.gas_limit,
            recipient: tx.recipient.unwrap(),
            amount: tx.amount,
            payload: tx.payload.clone(),
            signature: None,
        };
        signature.into_bytes()
    }
}


pub fn merkle_root_transactions(transactions: Vec<Transaction>) -> Hash {
    merkle_tree_root(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptocurrency_kit::ethkey::random::Random;
    use cryptocurrency_kit::ethkey::{Generator, KeyPair};
    use std::io::{self, Write};

    #[test]
    fn transaction_sign() {
        let keypair = Random.generate().unwrap();
        let mut tx = Transaction::new(10, Address::from(100), 89, 10, 90, vec![10, 39, 76, 31]);
        tx.sign(100, keypair.secret());
        let hash = tx.hash();
        writeln!(io::stdout(), "hash: {:?}", hash).unwrap();
        writeln!(io::stdout(), "{}", tx.pretty_json()).unwrap();
    }
}
