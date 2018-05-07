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


impl Database for MemoryDB{
    fn snapshot(&self) -> Box<Snapshot>{
        Box::new(MemoryDB{
            map: RwLock::new(self.map.read().unwrap().clone()),
        })
    }

    fn merge(&self, patch: Patch) Result<()> {
    }
}
