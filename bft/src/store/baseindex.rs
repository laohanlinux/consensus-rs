use std::{borrow::Cow, marker::PhantomData};
use std::io::Cursor;

use kvdb::DBValue;
use kvdb_rocksdb::{Database, DatabaseIterator};
use cryptocurrency_kit::storage::keys::StorageKey;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::{Hash, hash, CryptoHash};

use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

const COL: Option<u32> = None;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum IndexType {
    Entry,
    KeySet,
    List,
    SparseList,
    Map,
    ProofList,
    ProofMap,
    ValueSet,
}

impl From<u8> for IndexType {
    fn from(num: u8) -> Self {
        use self::IndexType::*;
        match num {
            0 => Entry,
            1 => KeySet,
            2 => List,
            3 => SparseList,
            4 => Map,
            5 => ProofList,
            6 => ProofMap,
            7 => ValueSet,
            invalid => panic!(
                "Unreachable pattern ({:?}) while constructing table type. \
                 Storage data is probably corrupted",
                invalid
            ),
        }
    }
}

implement_cryptohash_traits!(IndexType);
implement_storagevalue_traits!(IndexType);

pub struct BaseIndex{
    name: Vec<u8>,
    index_id: Option<Vec<u8>>,
    view: Database,
}

impl BaseIndex {

    fn prefix_key<K: StorageKey + ?Sized>(&self, key: &K) -> Vec<u8> {
        if let Some(ref prefix) = self.index_id {
            let mut v = vec![0; prefix.len() + key.size()];
            v[..prefix.len()].copy_from_slice(prefix);
            key.write(&mut v[prefix.len()..]);
            v
        }else {
            let mut v = vec![0; key.size()];
            key.write(&mut v);
            v
        }
    }

    pub fn get<K, V>(&self, key: &K) -> Option<V>
        where K: StorageKey + ?Sized,
            V: StorageValue,
    {
        let mut v = vec![0; self.name.len() + key.size()];
        v[..self.name.len()].copy_from_slice(&self.name);
        key.write(&mut v[self.name.len()..]);
        if let Some(value) = self.view.get(COL, &v).unwrap() {
            return Some(StorageValue::from_bytes(Cow::from(value.as_ref())));
        }
        None
    }

    pub fn contains<K>(&self, key: &K) -> bool
    where
        K: StorageKey + ?Sized,
    {
        self.view.get(COL,&self.long_key(key)).unwrap().is_some()
    }

    pub(crate) fn long_key<K>(&self, k: &K) -> Vec<u8>
    where K: StorageKey + ?Sized
    {
        let mut v = self.prefix_key(k);
        let mut buf = vec![0; self.name.len() + v.len()];
        buf[..self.name.len()].copy_from_slice(&self.name);
        buf[self.name.len()..].copy_from_slice(&v);
        buf
    }
}
