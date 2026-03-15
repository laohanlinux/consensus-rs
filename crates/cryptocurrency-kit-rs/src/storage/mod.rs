pub mod error;
pub mod hash;
#[macro_use]
pub mod keys;
#[macro_use]
pub mod values;

pub use self::error::Error;
pub use crate::encoding;

/// A specialized `Result` type for I/O operations with storage.
pub type Result<T> = ::std::result::Result<T, Error>;
