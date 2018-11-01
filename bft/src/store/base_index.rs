use std::io::Cursor;
use std::sync::Arc;
use std::{borrow::Cow, marker::PhantomData};

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::storage::keys::StorageKey;
use cryptocurrency_kit::storage::values::StorageValue;
use kvdb::{DBTransaction, DBValue};
use kvdb_rocksdb::{Database, DatabaseIterator};

use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use super::types::Iter;

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

pub struct BaseIndex {
    name: String,
    index_id: Option<Vec<u8>>,
    index_type: IndexType,
    view: Arc<Database>,
}

pub struct BaseIndexIter<'a, K, V> {
    base_iter: Iter<'a>,
    base_prefix_len: usize,
    index_id: Vec<u8>,
    ended: bool,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

impl BaseIndex {
    pub fn new<S: AsRef<str>>(index_name: S, index_type: IndexType, view: Arc<Database>) -> Self {
        Self {
            name: index_name.as_ref().to_string(),
            index_id: None,
            index_type,
            view,
        }
    }

    // Opz
    fn prefix_key<K: StorageKey + ?Sized>(&self, key: &K) -> Vec<u8> {
        let index_len = self.index_id.as_ref().map_or(0, |item| item.len());
        let name_len = self.name.len();
        let mut prefix_key = vec![0; name_len + index_len + key.size()];
        prefix_key[..name_len].copy_from_slice(self.name.as_bytes());

        if let Some(ref prefix) = self.index_id {
            prefix_key[name_len..index_len]
                .copy_from_slice(&self.index_id.as_ref().map_or(&vec![], |item| item));
        }

        key.write(&mut prefix_key[name_len + index_len..]);
        prefix_key
    }

    pub fn snapshot(&self) -> &Database {
        &self.view
    }

    pub fn get<K, V>(&self, key: &K) -> Option<V>
    where
        K: StorageKey + ?Sized,
        V: StorageValue,
    {
        let key = self.prefix_key(key);
        if let Some(value) = self.view.get(COL, &key).unwrap() {
            return Some(StorageValue::from_bytes(Cow::from(value.as_ref())));
        }
        None
    }

    pub fn contains<K>(&self, key: &K) -> bool
    where
        K: StorageKey + ?Sized,
    {
        self.view.get(COL, &self.prefix_key(key)).unwrap().is_some()
    }

    pub fn iter<P, K, V>(&self, subprefix: &P) -> BaseIndexIter<K, V>
    where
        P: StorageKey,
        K: StorageKey,
        V: StorageValue,
    {
        let iter_prefix = self.prefix_key(subprefix);
        BaseIndexIter {
            base_iter: self.view.iter_from_prefix(COL, &iter_prefix).unwrap(),
            base_prefix_len: self.name.len() + self.index_id.as_ref().map_or(0, |p| p.len()),
            index_id: iter_prefix,
            ended: false,
            _k: PhantomData,
            _v: PhantomData,
        }
    }

    //    pub fn iter_from<P, F, K, V>(&self, subprefix:&P, from:&F) -> BaseIndexIter<K, V>
    //    where
    //        P: StorageKey,
    //        F: StorageKey + ?Sized,
    //        K: StorageKey,
    //        V: StorageValue,
    //    {
    //        let iter_prefix = self.prefix_key(subprefix);
    //        let iter_from = self.prefix_key(from);
    //        BaseIndexIter {
    //            base_iter: self.view.iter_from_prefix(COL, )
    //        }
    //    }

    /////////////////////////////
    pub fn fork(&mut self) -> &Database {
        &self.view
    }

    pub fn transaction(&self) -> DBTransaction {
        self.view.transaction()
    }

    pub fn put_transaction(&self, tx: DBTransaction) {
        self.view.write(tx).unwrap();
        self.view.flush().unwrap();
    }

    pub fn put<K, V>(&mut self, key: &K, value: V)
    where
        K: StorageKey,
        V: StorageValue,
    {
        let key = self.prefix_key(key);
        let mut tx = self.view.transaction();
        tx.put_vec(COL, &key, value.into_bytes());
        self.view.write(tx).unwrap();
        self.view.flush().unwrap();
    }

    pub fn remove<K>(&mut self, key: &K)
    where
        K: StorageKey + ?Sized,
    {
        let key = self.prefix_key(key);
        let mut tx = self.view.transaction();
        tx.delete(COL, &key);
        self.view.flush().unwrap();
    }

    pub fn clear(&mut self) {
        let prefix = self.prefix_key("");
        if let Some(iter) = self.view.iter_from_prefix(COL, &prefix) {
            let mut tx = self.view.transaction();
            iter.for_each(|item| {
                tx.delete(COL, &item.0);
            });
            self.view.write(tx).unwrap();
            self.view.flush().unwrap();
        }
    }
}

impl<'a, K, V> Iterator for BaseIndexIter<'a, K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    type Item = (K::Owned, V);

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::{self, Write};
        if self.ended {
            return None;
        }

        if let Some((k, v)) = self.base_iter.next() {
            writeln!(io::stdout(), "=======>{:?}, {:?}", String::from_utf8_lossy(&k), v).unwrap();
            if k.starts_with(&self.index_id) {
                return Some((
                    K::read(&k[self.base_prefix_len..]),
                    V::from_bytes(Cow::Borrowed(&v)),
                ));
            }
        }
        self.ended = true;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    use common::random_dir;
    use rand::random;
    use std::borrow::Borrow;

    #[test]
    fn t() {
        let db = Arc::new(Database::open_default(&random_dir()).unwrap());
        {
            let index = BaseIndex::new("transaction", IndexType::Map, db.clone());
            let mut index = BaseIndex::new("transaction", IndexType::Map, db.clone());
            let prefix = "block_".to_string();
            (0..100).for_each(|idx| {
                let (key, value) = (format!("{}{}", prefix, idx), format!("{}", idx + 2));
                index.put(&key, value);
            });

            let iter = index.iter::<String, String, String>(&prefix);
            iter.for_each(|(key, value)| {
                writeln!(io::stdout(), "key:{}, value:{}", key, value).unwrap();
            })
        }
    }
//
//    #[test]
//    fn keys() {
//        let db = Arc::new(Database::open_default(&random_dir())).unwrap();
//        let mut index = BaseIndex::new("transaction", IndexType::Map, db.clone());
//        let prefix = "block_".to_string();
//        (0..100).for_each(|idx|{
//            let (key, value) = (format!("{}{}", prefix, idx), format!("{}", idx + 2));
//            index.put(&key, value);
//        });
//
//        let key_iter = index
//    }
}
