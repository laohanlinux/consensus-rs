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
use self::block::{Block};
use self::schema::{Schema};
use self::dpos::block::{Block as DposBlock};

use vec_map::VecMap;
use byteorder::{ByteOrder, LittleEndian};
use mount::Mount;
use failure;
use quick_protobuf::{Writer, Reader, MessageRead, MessageWrite};

use std::{fmt, iter, mem, panic};
use std::sync::Arc;
use std::collections::{BTreeMap, HashMap};
use std::error::Error as StdError;
use std::io::Cursor;
use std::borrow::Cow;

use messages::{RawMessage};
use crypto::{self, CryptoHash, Hash};
use storage::{Database, Error, Fork, Patch, Snapshot, StorageKey, StorageValue};
use helpers::{Height, Round, ValidatorId};
use encoding::Error as MessageError;


mod block;
//#[macro_use]
//mod transaction;
#[macro_use]
mod schema;

pub mod dpos;

/// Exonum blockchain instance with the concrete services set and data storage.
/// Only blockchains with the identical set of services and genesis block can be combined
/// into the single network.
pub struct Blockchain {
    db: Arc<Database>,
}

impl Blockchain {
    /// Constructs a blockchain for the given `storage` and list of `services`.
    pub fn new<D: Into<Arc<Database>>>(storage: D) -> Blockchain {
        Blockchain{
            db: storage.into(),
        }
    }

    /// Creates a readonly snapshot of the current storage state.
    pub fn snapshot(&self) -> Box<Snapshot> {self.db.snapshot()}

    /// Creates snapshot of the current state that can be later committed
    /// via `merge` method.
    pub fn fork(&self) -> Fork {self.db.fork()}

    /// Commits changes from the patch to the blockchain storage.
    /// See [`Fork`](../storage/struct.Fork.html) for details.
    pub fn merge(&mut self, patch: Patch) -> Result<(), Error> {self.db.merge(patch)}

}

impl fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Blockchain(..")
    }
}

impl Clone for Blockchain {
    fn clone(&self) -> Blockchain {
        Blockchain {
            db: Arc::clone(&self.db),
        }
    }
}

// TODO: use macro reimplements
impl StorageKey for DposBlock {

    fn size(&self) -> usize {
        self.get_size()
    }

    fn write(&self, buffer: &mut [u8]) {
        let mut writer = Writer::new(Cursor::new(buffer));
        self.write_message(&mut writer).unwrap();
    }

    fn read(buffer: &[u8]) -> Self::Owned{
        let mut reader = Reader::from_bytes(buffer.to_vec());
        reader.read(DposBlock::from_reader).unwrap()
    }
}

impl StorageValue for DposBlock {
    fn into_bytes(self) -> Vec<u8> {
        let capacity = self.get_size();
        let mut buffer = Vec::with_capacity(capacity);
        buffer.extend(iter::repeat(0).take(capacity));
        {
            let mut writer = Writer::new(&mut buffer);
            self.write_message(&mut writer).unwrap();
        }
        buffer
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        let mut reader = Reader::from_bytes(value.to_vec());
        reader.read(DposBlock::from_reader).unwrap()
    }
}

impl CryptoHash for DposBlock{
    fn hash(&self) -> Hash {
        let block_size = self.get_size();
        let mut buffer = Vec::with_capacity(block_size);
        buffer.extend(iter::repeat(0).take(block_size));
        self.write(&mut buffer);
        crypto::hash(&buffer)
    }
}


#[cfg(test)]
mod tests {

    use bytes::BufMut;
    use std::io::Cursor;
    use prost::Message;
    use storage::{StorageKey, StorageValue};
    use quick_protobuf::{Writer, Reader, MessageRead, MessageWrite};

    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};
    use rmp_serde::{Deserializer, Serializer};

    use std::io::{self, Write};
    use std::iter;

    use super::DposBlock as Block;

    #[test]
    fn test_storage_key_for_message_pack(){
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Human {
            age: u32,
            name: String,
        }

        let mut buf = vec![];
        assert_eq!(buf.len(), 0);
        let val = Human {
            age: 42,
            name: "John".into(),
        };

        val.serialize(&mut Serializer::new(&mut buf)).unwrap();
        assert!(buf.len() > 1);
    }

    #[test]
    fn test_storage_key_for_block(){
            let mut block = Block::default();
            block.height = 1_000;
            block.timestamp = 2_000;

            let block_size = block.get_size() -2;
            let mut buffer = Vec::with_capacity(block_size);
            buffer.extend(iter::repeat(0).take(block_size));
            block.write(&mut buffer);

            writeln!(io::stdout(), "{}", buffer[0]).unwrap();

            let new_block: Block = Block::read(&buffer);
            assert_eq!(new_block.height, block.height);
    }

}

