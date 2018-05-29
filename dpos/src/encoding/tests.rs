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

use bit_vec::BitVec;
use chrono::{Utc};
use byteorder::{ByteOrder, LittleEndian};
use uuid::Uuid;

use std::net::SocketAddr;

use crypto::{gen_keypair, hash};
use messages::{Message, RawMessage};
use helpers::{Height, Round, ValidatorId};
use super::{CheckedOffset, Field, Offset};

static VALIDATOR: ValidatorId = ValidatorId(65_123);
static HEIGHT: Height = Height(123_123_123);
static ROUND: Round = Round(321_321_312);

#[allow(dead_code)]
// This structures used to test deserialization,
// so we should ignore unused `new` method.
mod ignore_new {
    use crypto::Hash;
    encoding_struct! {
        struct Parent {
            child: Child,
        }
    }

    encoding_struct! {
        struct Child {
            child: &Hash,
        }
    }
}
