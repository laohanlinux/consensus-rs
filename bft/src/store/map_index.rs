use std::{borrow::Borrow, marker::PhantomData};

use cryptocurrency_kit::storage::{keys::StorageKey, values::StorageValue};
use kvdb_rocksdb::Database;

use super::base_index::{BaseIndex, BaseIndexIter, IndexType};

//#[derive(Debug)]
pub struct MapIndex<K, V> {
    base: BaseIndex,
    _k: PhantomData<K>,
    _v: PhantomData<V>,
}

pub struct MapIndexIter<'a, K, V> {
    base_iter: BaseIndexIter<'a, K, V>,
}

impl <'a, K, V> Iterator for MapIndexIter<'a, K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    type Item = (K::Owned, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.base_iter.next()
    }
}

//#[derive(Debug)]
pub struct MapIndexKeys<'a, K> {
    base_iter: BaseIndexIter<'a, K, ()>,
}

pub struct MapIndexValues<'a, V> {
    base_iter: BaseIndexIter<'a, (), V>,
}

impl<K, V> MapIndex<K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    pub fn new<S: AsRef<str>>(index_name: S, view: &'static Database) -> Self {
        Self {
            base: BaseIndex::new(index_name, IndexType::Map, view),
            _k: PhantomData,
            _v: PhantomData,
        }
    }

    //    pub fn new_in_family<S: AsRef<str>, I: StorageKey>(
    //        familly_name: S,
    //        index_id: &T,
    //        view: T,
    //    ) -> Self {
    //        Self {
    //
    //        }
    //    }

    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: StorageKey + ?Sized,
    {
        self.base.get(key)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: StorageKey + ?Sized,
    {
        self.base.contains(key)
    }

    pub fn iter(&self) -> MapIndexIter<K, V> {
        MapIndexIter {
            base_iter: self.base.iter(&()),
        }
    }

    pub fn keys(&self) -> MapIndexKeys<K> {
        MapIndexKeys {
            base_iter: self.base.iter(&()),
        }
    }

    pub fn values(&self) -> MapIndexValues<V> {
        MapIndexValues {
            base_iter: self.base.iter(&()),
        }
    }
}

impl<K, V> MapIndex<K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    pub fn put(&mut self, key: &K, value: V) {
        self.base.put(key, value)
    }

    pub fn remove<Q>(&mut self, key: &Q)
    where
        K: Borrow<Q>,
        Q: StorageKey + ?Sized,
    {
        self.base.remove(key)
    }

    pub fn clear(&mut self) {
        self.base.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::random_dir;
    use std::io::{self, Write};

    const IDX_NAME: &'static str = "idx_name";
    #[test]
    fn str_key() {
        let db = Database::open_default(&random_dir()).unwrap();
        const KEY: &str = "key_1";
        db.borrow()
        let mut index: MapIndex<String, _> = MapIndex::new(IDX_NAME, &db);
        assert_eq!(false, index.contains(KEY));
        index.put(&KEY.to_owned(), 0);
        assert_eq!(true, index.contains(KEY));
    }

    #[test]
    fn map_iter() {
        let db = Database::open_default(&random_dir()).unwrap();

        {
            let mut index: MapIndex<String, _> = MapIndex::new(IDX_NAME, &db);

            (0..100).for_each(|idx|{
                index.put(&format!("{}", idx), idx+1);
            });

            let iter = index.iter();
            iter.for_each(|(key, value)|{
                writeln!(io::stdout(), "key: {}, value: {}", key, value);
            });

            let iter = index.iter();
            assert_eq!(iter.count(), 100);
        }
    }
}
