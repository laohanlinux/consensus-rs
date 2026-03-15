use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use secp256k1::constants::SECRET_KEY_SIZE as SECP256K1_SECRET_KEY_SIZE;
use secp256k1::key;
use ethereum_types::H256;
use crate::mem::Memzero;

use super::{Error, SECP256K1};
use crate::common::to_fixed_array_32;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret {
    inner: Memzero<H256>,
}

impl fmt::LowerHex for Secret {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(fmt)
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(fmt)
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Secret: 0x{:x}{:x}..{:x}{:x}", self.inner[0], self.inner[1], self.inner[30], self.inner[31])
    }
}

impl Secret {
    /// Creates a `Secrect` from the given slice, returning `None` if the slice length != 32
    pub fn from_slice(key: &[u8]) -> Option<Self> {
        if key.len() != 32 {
            return None;
        }
        // TODO Opz use unsafe code to advoice alloc new space
        let h = H256::from(to_fixed_array_32(key));
        Some(Secret { inner: Memzero::from(h) })
    }

    /// Create zero key, which is invalid for crypto operation, but valid for math operation.
    pub fn zero() -> Self {
        Secret { inner: Memzero::from(H256::default()) }
    }

    /// Imports and validates the key
    pub fn from_unsafe_slice(key: &[u8]) -> Result<Self, Error> {
        let secrect = key::SecretKey::from_slice(&super::SECP256K1, key)?;
        Ok(secrect.into())
    }

    /// Checks validity of this key
    pub fn check_validity(&self) -> Result<(), Error> {
        self.to_secp256k1_secret().map(|_| ())
    }


    /// Create `secp256k1::key::SecretKey` based on this secret
    pub fn to_secp256k1_secret(&self) -> Result<key::SecretKey, Error> {
        Ok(key::SecretKey::from_slice(&SECP256K1, &self[..])?)
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", *self.inner)
    }
}


impl FromStr for Secret {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(H256::from_str(s).map_err(|e| Error::Custom(format!("{:?}", e)))?.into())
    }
}

impl From<[u8; 32]> for Secret {
    fn from(k: [u8; 32]) -> Self {
        let inner = Memzero::from(H256::from(k));
        Secret { inner: inner }
    }
}

impl From<H256> for Secret {
    fn from(s: H256) -> Self {
        Secret {
            inner: Memzero::from(s),
        }
    }
}

impl From<&'static str> for Secret {
    fn from(s: &'static str) -> Self {
        s.parse().expect(&format!("Invalid string literal for {}: '{}'", stringify!(Self), s))
    }
}

impl From<key::SecretKey> for Secret {
    fn from(key: key::SecretKey) -> Self {
        let mut a = [0; SECP256K1_SECRET_KEY_SIZE];
        a.copy_from_slice(&key[0..SECP256K1_SECRET_KEY_SIZE]);
        a.into()
    }
}

impl Deref for Secret {
    type Target = H256;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
