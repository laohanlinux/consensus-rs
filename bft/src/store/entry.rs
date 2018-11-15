use std::marker::PhantomData;
use std::sync::Arc;

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::storage::keys::StorageKey;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::types::Zero;
use kvdb_rocksdb::Database;

use super::base_index::{BaseIndex, BaseIndexIter, IndexType};

#[derive(Debug)]
pub struct Entry<V> {
    base: BaseIndex,
    _v: PhantomData<V>,
}

impl<V> Entry<V>
where
    V: StorageValue,
{
    pub fn new<S: AsRef<str>>(index_name: S, view: Arc<Database>) -> Self {
        Entry {
            base: BaseIndex::new(index_name, IndexType::Entry, view),
            _v: PhantomData,
        }
    }

    pub fn get(&self) -> Option<V> {
        self.base.get(&Zero)
    }

    pub fn exists(&self) -> bool {
        self.base.contains(&Zero)
    }

    pub fn hash(&self) -> Hash {
        self.base
            .get::<Zero, V>(&Zero)
            .map(|v| v.hash())
            .unwrap_or_default()
    }
    //////////
    pub fn set(&mut self, value: V) {
        self.base.put(&Zero, value)
    }

    pub fn remove(&mut self) {
        self.base.remove(&Zero)
    }

    pub fn take(&mut self) -> Option<V> {
        let value: Option<V> = self.get();
        if value.is_some() {
            self.remove();
        }
        value
    }

    pub fn swap(&mut self, value: V) -> Option<V> {
        let pre_value = self.get();
        self.set(value);
        pre_value
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use super::*;
    use common::random_dir;
    use cryptocurrency_kit::crypto::EMPTY_HASH;

    #[test]
    fn entry() {
        let mut entry: Entry<i32> = Entry::new(
            "IDX_NAME",
            Arc::new(Database::open_default(&random_dir()).unwrap()),
        );

        {
            assert_eq!(entry.get().is_none(), true);
            assert_eq!(entry.exists(), false);
            assert_eq!(entry.hash(), EMPTY_HASH);
        }

        {
            entry.set(10);
            assert_eq!(entry.get(), Some(10_i32));
            assert_eq!(entry.exists(), true);
            writeln!(io::stdout(), "10_i32_entry => {:?}", entry.hash()).unwrap();
        }

        {
            entry.remove();
            assert_eq!(entry.get().is_none(), true);
            assert_eq!(entry.exists(), false);
            assert_eq!(entry.hash(), EMPTY_HASH);
        }
    }
}
