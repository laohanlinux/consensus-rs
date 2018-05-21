use dposblock::Block;

use storage::{StorageKey, StorageValue};
use prost::Message;

impl StorageKey for Block {
    fn size(&self) -> usize {
        self.encoded_len()
    }

    // TODO: Opz
    fn write(&self, buffer: &mut [u8]) {
        let mut buf = Vec::with_capacity(self.encoded_len());
        self.encode(&mut buf).unwrap();
        buffer.copy_from_slice(&buf);
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        Block::decode(buffer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use std::io::Cursor;
    use prost::Message;
    use storage::{StorageKey, StorageValue};
    use dposblock::Block;
    use quick_protobuf::{Reader, MessageRead, BytesReader, Writer};

    use std::io::{self, Write};

    #[test]
    fn test_storage_key_for_block(){

        {
            let mut block = Block::default();
            block.height = 1_000;
            block.timestamp = 2_000;

            let mut buffer = [0u8; 6];
            block.write(&mut buffer);

            let new_block: Block = Block::read(&buffer);
            assert_eq!(new_block.height, block.height);
        }

        {
            let block = Block{height:100, timestamp: 129};
            let mut bytes: Vec<u8>;
            {
                let mut writer = Writer::new(&mut bytes);
                writer.write_message(&block).expect("Cannot write message!");
            }
            writeln!(io::stdout(), "{}", bytes[0]);

        }


    }

}

