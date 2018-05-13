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

//! Consensus and other messages and related utilities.

pub use self::raw::{Message, MessageBuffer, MessageWriter, RawMessage, ServiceMessage,
                    HEADER_LENGTH, PROTOCOL_MAJOR_VERSION};

use bit_vec::BitVec;

use std::fmt;

use crypto::PublicKey;
use encoding::Error;
use helpers::{Height, Round, ValidatorId};

#[macro_use]
mod raw;

#[cfg(test)]
mod tests;

// TODO: implement common methods for enum types (hash, raw, from_raw, verify)
// TODO: use macro for implementing enums (ECR-166)

/// Raw transaction type.
pub type RawTransaction = RawMessage;

impl fmt::Debug for RawTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("version", &self.version())
            .field("service_id", &self.service_id())
            .field("message_type", &self.message_type())
            .field("length", &self.len())
            .field("hash", &self.hash())
            .finish()
    }
}