use std::borrow::Cow;
use std::io;

use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use cryptocurrency_kit::storage::values::StorageValue;
use tokio::codec::{Decoder, Encoder};

use super::protocol::*;

pub const MAX_MSG_SIZE: u32 = 1 << 10;
pub const MSG_SIZE: u32 = 4; // byte

// |msg_size: 4bytes| msg encode |
pub struct MsgPacketCodec;

impl Decoder for MsgPacketCodec {
    type Item = RawMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < MSG_SIZE as usize {
                // continue read
                return Ok(None);
            }
            BigEndian::read_u32(src.as_ref())
        };

        if src.len() >= (size + MSG_SIZE) as usize {
            src.split_to(MSG_SIZE as usize);
            let buf = src.split_to(size as usize);
            let raw_message: RawMessage = RawMessage::from_bytes(Cow::from(buf.to_vec()));
            Ok(Some(raw_message))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for MsgPacketCodec {
    type Item = RawMessage;
    type Error = io::Error;

    fn encode(&mut self, msg: RawMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = msg.into_bytes();
        let size = msg.len() as u32;
        dst.reserve((size + MSG_SIZE) as usize);
        dst.put_u32_be(size);
        dst.put(msg);
        Ok(())
    }
}
