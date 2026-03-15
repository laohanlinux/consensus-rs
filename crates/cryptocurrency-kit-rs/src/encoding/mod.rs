#[macro_use]
pub mod json;
#[macro_use]
pub mod bigarray;

/// use third package alias local package
pub use hex::{FromHex, FromHexError};