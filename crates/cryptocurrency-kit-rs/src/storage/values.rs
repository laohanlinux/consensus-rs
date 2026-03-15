// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A definition of `StorageValue` trait and implementations for common types.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use std::borrow::Cow;

use super::hash::UniqueHash;
use crate::types::Zero;
use crate::crypto::Hash;
use crate::ethkey::Public as PublicKey;

pub trait StorageValue: UniqueHash + Sized {
    /// Serialize a value into a vector of bytes.
    fn into_bytes(self) -> Vec<u8>;

    /// Deserialize a value from bytes.
    fn from_bytes(value: Cow<[u8]>) -> Self;
}

#[macro_export]
macro_rules! implement_storagevalue_traits {
    ($key: ident) => {
        impl StorageValue for $key {
            fn into_bytes(self) -> Vec<u8> {
                serde_json::to_vec(&self).unwrap()
            }
            fn from_bytes(value: Cow<[u8]>) -> Self {
                serde_json::from_slice(&value).unwrap()
            }
        }
    };
}

implement_storagevalue_traits! {bool}
implement_storagevalue_traits! {u8}
implement_storagevalue_traits! {u16}
implement_storagevalue_traits! {u32}
implement_storagevalue_traits! {u64}
implement_storagevalue_traits! {i8}
implement_storagevalue_traits! {i16}
implement_storagevalue_traits! {i32}
implement_storagevalue_traits! {i64}
// Uses UTF-8 string serialization.
implement_storagevalue_traits! {String}
implement_storagevalue_traits! {Uuid}

/// No-op implementation.
impl StorageValue for () {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

impl StorageValue for Zero {
    fn into_bytes(self) -> Vec<u8> {
        vec![]
    }

    fn from_bytes(_: Cow<[u8]>) -> Self {
        Zero
    }
}

// Hash is very special
impl StorageValue for Hash {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

impl StorageValue for PublicKey {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

//impl StorageValue for RawMessage {
//    fn into_bytes(self) -> Vec<u8> {
//        self.as_ref().to_vec()
//    }
//
//    fn from_bytes(value: Cow<[u8]>) -> Self {
//        Self::new(MessageBuffer::from_vec(value.into_owned()))
//    }
//}

impl StorageValue for Vec<u8> {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

/// Uses little-endian encoding.
impl StorageValue for DateTime<Utc> {
    fn into_bytes(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero(){
        let zero1 = Zero::from_bytes(Cow::from(vec![]));
        assert_eq!(0, zero1.into_bytes().len());
    }
}
