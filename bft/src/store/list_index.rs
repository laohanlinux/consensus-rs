use std::sync::Arc;
use std::{cell::Cell, marker::PhantomData};

use cryptocurrency_kit::crypto::*;
use cryptocurrency_kit::storage::{keys::StorageKey, values::StorageValue};
use cryptocurrency_kit::types::Zero;
use kvdb_rocksdb::{Database, DatabaseIterator};

use super::base_index::{BaseIndex, BaseIndexIter, IndexType};

/// data format
/// |length|l-0, l-1, l-2, l-3|
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u64 {
        if let Some(len) = self.length.get() {
            return len;
        }
        let len = self.base.get(&Zero).unwrap_or(0);
        self.length.set(Some(len));
        len
    }

    pub fn iter(&self) -> ListIndexIter<V> {
        ListIndexIter {
            base_iter: self.base.iter_from(&Zero, &0_u64),
        }
    }

    pub fn iter_from(&self, from: u64) -> ListIndexIter<V> {
        ListIndexIter {
            base_iter: self.base.iter_from(&Zero, &from),
        }
    }

    /// mut
    pub fn set_len(&mut self, len: u64) {
        self.base.put(&Zero, len);
        self.length.set(Some(len));
    }

    pub fn push(&mut self, value: V) {
        let len = self.len();
        self.base.put(&len, value);
        self.set_len(len + 1)
    }

    pub fn pop(&mut self) -> Option<V> {
        match self.len() {
            0 => None,
            l => {
                let v = self.base.get(&(l - 1));
                self.base.remove(&(l - 1));
                self.set_len(l - 1);
                v
            }
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = V>,
    {
        use std::io::{self, Write};
        let mut len = self.len();
        for value in iter {
            self.base.put(&len, value);
            len += 1;
        }
        self.base.put(&Zero, len);
        self.set_len(len);
    }

    pub fn truncate(&mut self, len: u64) {
        while self.len() > len {
            self.pop();
        }
    }

    pub fn set(&mut self, index: u64, value: V) {
        if index >= self.len() {
            panic!(
                "index out of bound: \
                 the len is {} but the index is {}",
                self.len(),
                index,
            );
        }
        self.base.put(&index, value);
    }

    pub fn clear(&mut self) {
        self.length.set(Some(0));
        self.base.clear();
    }
}

impl<'a, V> ::std::iter::IntoIterator for &'a ListIndex<V>
where
    V: StorageValue,
{
    type Item = V;
    type IntoIter = ListIndexIter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, V> Iterator for ListIndexIter<'a, V>
where
    V: StorageValue,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        self.base_iter.next().map(|(.., v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    fn list_index_methods(list_index: &mut ListIndex<i32>) {
        assert!(list_index.is_empty());
        assert_eq!(0, list_index.len());
        assert!(list_index.last().is_none());
        assert_eq!(None, list_index.pop());

        let extended_by = vec![45, 3422, 234];
        list_index.extend(extended_by);
        assert!(!list_index.is_empty());
        assert_eq!(Some(45), list_index.get(0));
        assert_eq!(Some(3422), list_index.get(1));
        assert_eq!(Some(234), list_index.get(2));
        assert_eq!(3, list_index.len());

        list_index.set(2, 777);
        assert_eq!(Some(777), list_index.get(2));
        assert_eq!(Some(777), list_index.last());
        assert_eq!(3, list_index.len());

        let mut extended_by_again = vec![666, 999];
        for el in &extended_by_again {
            list_index.push(*el);
        }
        assert_eq!(Some(666), list_index.get(3));
        assert_eq!(Some(999), list_index.get(4));
        assert_eq!(5, list_index.len());
        extended_by_again[1] = 1001;
        list_index.extend(extended_by_again);
        assert_eq!(7, list_index.len());
        assert_eq!(Some(1001), list_index.last());

        assert_eq!(Some(1001), list_index.pop());
        assert_eq!(6, list_index.len());

        list_index.truncate(3);

        assert_eq!(3, list_index.len());
        assert_eq!(Some(777), list_index.last());

        list_index.clear();
        assert_eq!(0, list_index.len());
    }

    fn list_index_iter(list_index: &mut ListIndex<u8>) {

        list_index.extend(vec![1u8, 2, 3]);

        assert_eq!(list_index.len(), 3);
        assert_eq!(list_index.get(0).unwrap(), 1);
        assert_eq!(list_index.get(1).unwrap(), 2);
        assert_eq!(list_index.get(2).unwrap(), 3);
        assert_eq!(list_index.last().unwrap(), 3);
        assert_eq!(list_index.iter().collect::<Vec<u8>>(), vec![1, 2, 3]);
        assert_eq!(list_index.iter_from(0).collect::<Vec<u8>>(), vec![1, 2, 3]);
        assert_eq!(list_index.iter_from(1).collect::<Vec<u8>>(), vec![2, 3]);
        assert_eq!(
            list_index.iter_from(3).collect::<Vec<u8>>(),
            Vec::<u8>::new()
        );
    }
    fn newdb() -> Database {
        use crate::common::random_dir;
        Database::open_default(&random_dir()).unwrap()
    }
    mod rocksdb_tests {
        use super::*;
        const IDX_NAME: &'static str = "idx_name";

        #[test]
        fn test_list_index_methods() {
            let db = Arc::new(newdb());
            let mut list_index = ListIndex::new(IDX_NAME, db.clone());
            super::list_index_methods(&mut list_index);
        }

        #[test]
        fn test_list_index_iter(){
            let db = Arc::new(newdb());
            let mut list_index = ListIndex::new(IDX_NAME, db.clone());
            super::list_index_iter(&mut list_index);
        }
    }
}
