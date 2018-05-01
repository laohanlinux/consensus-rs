pub use self::error::Error;

/// A specialized `Result` type for I/O operations
pub type Result<T> = ::std::result::Result<T, Error>;

mod error;
mod db;