use std::sync::Arc;
use std::{borrow::Borrow, marker::PhantomData};

use cryptocurrency_kit::storage::{keys::StorageKey, values::StorageValue};
use cryptocurrency_kit::types::Zero;
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

impl<'a, K, V> Iterator for MapIndexIter<'a, K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    type Item = (K::Owned, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.base_iter.next()
    }
}

//#[derive(Debug, Clone)]
pub struct MapIndexKeys<'a, K> {
    base_iter: BaseIndexIter<'a, K, Zero>,
}

pub struct MapIndexValues<'a, V> {
    base_iter: BaseIndexIter<'a, Zero, V>,
}

impl<K, V> MapIndex<K, V>
where
    K: StorageKey,
    V: StorageValue,
{
    pub fn new<S: AsRef<str>>(index_name: S, view: Arc<Database>) -> Self {
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

impl<'a, K> Iterator for MapIndexKeys<'a, K>
where
    K: StorageKey,
{
    type Item = K::Owned;

    fn next(&mut self) -> Option<Self::Item> {
        // ignore the value
        self.base_iter.next().map(|(k, ..)| k)
    }
}

impl<'a, V> Iterator for MapIndexValues<'a, V>
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
    use common::random_dir;
    use std::io::{self, Write};

    const IDX_NAME: &'static str = "idx_name_";

    fn newdb() -> Database {
        Database::open_default(&random_dir()).unwrap()
    }

    #[test]
    fn str_key() {
        let db = Arc::new(newdb());
        const KEY: &str = "key_1";
        let mut index: MapIndex<String, _> = MapIndex::new(IDX_NAME, db);
        assert_eq!(false, index.contains(KEY));
        index.put(&KEY.to_owned(), 0);
        assert_eq!(true, index.contains(KEY));
    }

    #[test]
    fn key_iter() {
        let db = Arc::new(newdb());
        let mut index: MapIndex<String, String> = MapIndex::new(IDX_NAME, db.clone());
        let mut keys = index.keys();
        assert_eq!(keys.count(), 0);

        (0..100).for_each(|idx| {
            index.put(&format!("{}", idx), (idx + 1).to_string());
        });
        let ref mut keys = index.keys();
        assert_eq!(keys.count(), 100);

        let ref mut keys = index.keys();
        keys.for_each(|key| {
            writeln!(io::stdout(), "key: {}", key).unwrap();
        });
    }

    #[test]
    fn value_iter() {
        let db = Arc::new(newdb());
        let mut index: MapIndex<String, String> = MapIndex::new(IDX_NAME, db.clone());
        assert_eq!(index.values().count(), 0);

        (0..100).for_each(|idx| {
            index.put(&format!("{}", idx), (idx + 1).to_string());
        });
        assert_eq!(index.values().count(), 100);

        let ref mut values = index.values();
        values.for_each(|value| {
            writeln!(io::stdout(), "value: {}", value).unwrap();
        });
    }

    #[test]
    fn map_iter() {
        let db = Arc::new(newdb());

        {
            let mut index: MapIndex<String, i32> = MapIndex::new(IDX_NAME, db.clone());

            (0..100).for_each(|idx| {
                index.put(&format!("{}", idx), idx + 1);
            });
            let mut keys = index.keys();
            assert_eq!(keys.count(), 100);
        }

        {
            let mut index: MapIndex<String, _> =
                MapIndex::new("index2_name".to_string(), db.clone());
            (0..100).for_each(|idx| {
                index.put(&format!("{}", idx), idx + 1);
            });

            let iter = index.iter();
            iter.for_each(|(key, value)| {
                writeln!(io::stdout(), "key: {}, value: {}", key, value).unwrap();
            });

            let iter = index.iter();
            assert_eq!(iter.count(), 100);
        }
    }
}
