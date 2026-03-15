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

#![allow(unsafe_code)]

//! A definition of `StorageKey` trait and implementations for common types.
use crate::crypto::{HASH_SIZE, Hash};
use crate::ethkey::{Public, SIGNATURE_SIZE, Signature};
use crate::types::Zero;

use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

pub trait StorageKey: ToOwned {
    /// Returns the size of the serialized key in bytes.
    fn size(&self) -> usize;

    /// Serializes the key into the specified buffer of bytes.
    ///
    /// The caller must guarantee that the size of the buffer is equal to the precalculated size
    /// of the serialized key.
    // TODO: Should be unsafe? (ECR-174)
    fn write(&self, buffer: &mut [u8]);

    /// Deserializes the key from the specified buffer of bytes.
    // TODO: Should be unsafe? (ECR-174)
    fn read(buffer: &[u8]) -> Self::Owned;
}

/// No-op implementation.
impl StorageKey for Zero {
    fn size(&self) -> usize {
        0
    }

    fn write(&self, _: &mut [u8]) {
        // no-op
    }

    fn read(_: &[u8]) -> Self::Owned {
        Zero
    }
}

impl StorageKey for () {
    fn size(&self) -> usize {
        0
    }

    fn write(&self, _buffer: &mut [u8]) {
        // no-op
    }

    fn read(_buffer: &[u8]) -> Self::Owned {
        ()
    }
}

impl StorageKey for u8 {
    fn size(&self) -> usize {
        1
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer[0] = *self
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        buffer[0]
    }
}

/// Uses encoding with the values mapped to `u8`
/// by adding the corresponding constant (`128`) to the value.
impl StorageKey for i8 {
    fn size(&self) -> usize {
        1
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer[0] = self.wrapping_add(i8::min_value()) as u8;
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        buffer[0].wrapping_sub(i8::min_value() as u8) as i8
    }
}

/// Uses UTF-8 string serialization.
impl StorageKey for String {
    fn size(&self) -> usize {
        self.len()
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.as_bytes())
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        unsafe { ::std::str::from_utf8_unchecked(buffer).to_string() }
    }
}

impl StorageKey for str {
    fn size(&self) -> usize {
        self.len()
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.as_bytes())
    }

    // TODO FIXME
    fn read(buffer: &[u8]) -> Self::Owned {
        String::from_utf8_lossy(buffer).to_string()
    }
}

/// `chrono::DateTime` uses only 12 bytes in the storage. It is represented by number of seconds
/// since `1970-01-01 00:00:00 UTC`, which are stored in the first 8 bytes as per the `StorageKey`
/// implementation for `i64`, and nanoseconds, which are stored in the remaining 4 bytes as per
/// the `StorageKey` implementation for `u32`.
impl StorageKey for DateTime<Utc> {
    fn size(&self) -> usize {
        12
    }

    fn write(&self, buffer: &mut [u8]) {
        let secs = self.timestamp();
        let nanos = self.timestamp_subsec_nanos();
        secs.write(&mut buffer[0..8]);
        nanos.write(&mut buffer[8..12]);
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        let secs = i64::read(&buffer[0..8]);
        let nanos = u32::read(&buffer[8..12]);
        Utc.timestamp_opt(secs, nanos).single().unwrap_or_else(Utc::now)
    }
}

impl StorageKey for Uuid {
    fn size(&self) -> usize {
        16
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.as_bytes());
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        Uuid::from_bytes(buffer).unwrap()
    }
}

impl StorageKey for Decimal {
    fn size(&self) -> usize {
        16
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.serialize());
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(buffer);
        Self::deserialize(bytes)
    }
}

macro_rules! storage_key_for_ints {
    ($utype:ident, $itype:ident, $size:expr, $read_method:ident, $write_method:ident) => {
        /// Uses big-endian encoding.
        impl StorageKey for $utype {
            fn size(&self) -> usize {
                $size
            }

            fn write(&self, buffer: &mut [u8]) {
                BigEndian::$write_method(buffer, *self);
            }

            fn read(buffer: &[u8]) -> Self {
                BigEndian::$read_method(buffer)
            }
        }

        /// Uses big-endian encoding with the values mapped to the unsigned format
        /// by adding the corresponding constant to the value.
        impl StorageKey for $itype {
            fn size(&self) -> usize {
                $size
            }

            fn write(&self, buffer: &mut [u8]) {
                BigEndian::$write_method(buffer, self.wrapping_add($itype::min_value()) as $utype);
            }

            fn read(buffer: &[u8]) -> Self {
                BigEndian::$read_method(buffer).wrapping_sub($itype::min_value() as $utype)
                    as $itype
            }
        }
    };
}

storage_key_for_ints! {u16, i16, 2, read_u16, write_u16}
storage_key_for_ints! {u32, i32, 4, read_u32, write_u32}
storage_key_for_ints! {u64, i64, 8, read_u64, write_u64}

#[macro_export]
macro_rules! storage_key_for_crypto_types {
    ($type:ident, $size:expr) => {
        impl StorageKey for $type {
            fn size(&self) -> usize {
                $size
            }

            fn write(&self, buffer: &mut [u8]) {
                buffer.copy_from_slice(self.as_ref())
            }

            fn read(buffer: &[u8]) -> Self {
                $type::from_slice(buffer)
            }
        }
    };
}

#[macro_export]
macro_rules! storage_key_for_crypto_option_types {
    ($type:ident, $size:expr) => {
        impl StorageKey for $type {
            fn size(&self) -> usize {
                $size
            }

            fn write(&self, buffer: &mut [u8]) {
                buffer.copy_from_slice(self.as_ref())
            }

            fn read(buffer: &[u8]) -> Self {
                match $type::from_slice(buffer) {
                    Some(n) => n,
                    None => $type::default(),
                }
            }
        }
    };
}

storage_key_for_crypto_types! {Signature, SIGNATURE_SIZE}
storage_key_for_crypto_types! {Public, 64}
storage_key_for_crypto_option_types! {Hash, HASH_SIZE}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ethkey::{Generator, Random};
    use std::io::{self, Write};

    #[test]
    fn mannul() {
        let keypair = Random.generate().unwrap();
    }
}
