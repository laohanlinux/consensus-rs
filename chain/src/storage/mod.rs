pub use self::error::Error;

/// A specialized `Result` type for I/O operations
pub type Result<T> = ::std::result::Result<T, Error>;

pub use self::db::{Change, Changes, Database, Fork, Iter, Iterator, Patch,
    PatchIterator, Snapshot};

pub use self::memorydb::MemoryDB;

mod error;
mod db;
mod memorydb;
mod hash;