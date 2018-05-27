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

//! The module containing building blocks for creating blockchains powered by
//! the Exonum framework.
//!
//! Services are the main extension point for the Exonum framework. To create
//! your service on top of Exonum blockchain you need to do the following:
//!
//! - Define your own information schema.
//! - Create one or more transaction types using the [`transactions!`] macro and
//!   implement the [`Transaction`] trait for them.
//! - Create a data structure implementing the [`Service`] trait.
//! - Write API handlers for the service, if required.
//!
//! You may consult [the service creation tutorial][doc:create-service] for a more detailed
//! manual on how to create services.
//!
//! [`transactions!`]: ../macro.transactions.html
//! [`Transaction`]: ./trait.Transaction.html
//! [`Service`]: ./trait.Service.html
//! [doc:create-service]: https://exonum.com/doc/get-started/create-service
pub use self::block::{Block, SCHEMA_MAJOR_VERSION, BlockProof};
pub use self::schema::Schema;
pub use self::transaction::{Transaction, TransactionResult};
pub use self::dpos::block::Block as DposBlock;
pub use self::genesis::GenesisConfig;

use vec_map::VecMap;
use byteorder::{ByteOrder, LittleEndian};
use mount::Mount;
use failure;
use quick_protobuf::{Writer, Reader, MessageRead, MessageWrite};

use messages::RawMessage;
use crypto::{self, CryptoHash, Hash};
use storage::{Database, Error, Fork, Patch, Snapshot, StorageKey, StorageValue};
use helpers::{Height, Round, ValidatorId};
use encoding::Error as MessageError;


mod block;
#[macro_use]
mod transaction;
#[macro_use]
mod schema;
mod dpos;
mod genesis;
