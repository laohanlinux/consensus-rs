use std::sync::Arc;
use std::{cell::Cell, marker::PhantomData};

use cryptocurrency_kit::crypto::*;
use cryptocurrency_kit::storage::{keys::StorageKey, values::StorageValue};
use cryptocurrency_kit::types::Zero;
use kvdb_rocksdb::{Database, DatabaseIterator};

use super::base_index::{BaseIndex, BaseIndexIter, IndexType};

/// data format
/// |length|l-0l-1|l-2|l-3|
/// example: index_name: "IDX_NAME"
/// length: IDX_NAME,
/// list0: IDX_NAME0
/// list1: IDX_NAME1,
/// ListX: IDX_NAMEX,

#[debug]
pub struct ListIndex<V> {
    base: BaseIndex,
    length: Cell<Option<u64>>,
    _v: PhantomData<V>,
}

#[debug]
pub struct ListIndexIter<'a, V> {
    base_iter: BaseIndexIter<'a, u64, V>,
}

impl<V> ListIndex<V>
where
    V: StorageValue,
{
    pub fn new<S: AsRef<str>>(index_name: S, view: Arc<Database>) -> Self {
        Self {
            base: BaseIndex::new(index_name, IndexType::List, view),
            length: Cell::new(None),
            _v: PhantomData,
        }
    }

    pub fn get(&self, index: u64) -> Option<V>
    where
        V: StorageValue,
    {
        self.base.get(&index)
    }

    pub fn last(&self) -> Option<V>
    where
        V: StorageValue,
    {
        match self.len() {
            0 => None,
            l => self.get(l - 1),
        }
    }

    pub fn len(&self) -> u64 {
        if let Some(len) = self.length.get() {
            return len;
        }
        let len = self.base.get(&Zero).unwrap_or(0);
        self.length.set(Some(len));
        len
    }

    //  pub fn iter(&self) -> ListIndexIter<V> {
    //      ListIndexIter {
    //          base_iter: self.base.iter(&Zero, &0_u64),
    //      }
    //  }
}
