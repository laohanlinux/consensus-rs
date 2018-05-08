///! An implementation of `MemoryDB` database.
use std::sync::{Arc, RwLock};
use std::clone::Clone;
use std::collections::{BTreeMap, HashMap};

use super::{Database, Iter, Iterator, Patch, Result, Snapshot};
use super::db::Change;

type DB = HashMap<String, BTreeMap<Vec<u8>, Vec<u8>>>;


#[derive(Dafault, Debug)]
pub struct MemoryDB {
    map: RwLock<DB>,
}

/// An iterator over the entries of a `MemoryDB`.
struct MemoryDBIterator{
    data: Vec<Vec<u8>, Vec<u8>>,
    index: usize,
}

impl Database for MemoryDB{
    fn snapshot(&self) -> Box<Snapshot>{
        Box::new(MemoryDB{
            map: RwLock::new(self.map.read().unwrap().clone()),
        })
    }

    fn merge(&self, patch: Patch) ->Result<()> {
        let mut guard = self.map.write().unwrap();
        for (cf_name, changes) in patch {
            // iter all changes
            if !guard.contains_key(&cf_name) {
                guard.insert(cf_name.clone(), BTreeMap::new());
            }
            let table = guard.get_mut(&cf_name).unwrap();
            for (key, change) in changes {
                match change {
                    Change::Put(ref value) => {
                        table.insert(key, value.to_vec());
                    }
                    Change::Delete => {
                        table.remove(&key);
                    }
                }
            }
        }
        Ok(())
    }

    fn merge_sync(&self, patch: Patch) -> Result<()> {
        self.merge(patch)
    }
}


impl Snapshot for MemoryDB {
    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>> {
        self.map
            .read()
            .unwrap()
            .get(name)
            .and_then(|table|table.get(key).cloned())
    }

}