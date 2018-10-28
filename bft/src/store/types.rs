use kvdb::DBTransaction;
use kvdb_rocksdb::DatabaseIterator;

pub type Iter<'a> = DatabaseIterator<'a>;


pub trait Snapshot: 'static {
    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>>;

    fn contains(&self, name: &str, key: &[u8]) -> bool {
        self.get(name, key).is_some()
    }

    fn iter<'a> (&'a self, name: &str, from: &[u8]) -> Iter<'a>;
}

pub struct Fork {
    snapshot: Box<dyn Snapshot>,
    transaction: DBTransaction,
}

//
//impl Snapshot for Fork {
//    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>> {
//
//    }
//}