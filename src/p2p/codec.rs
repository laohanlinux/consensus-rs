use std::borrow::Cow;
use std::io;

use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use cryptocurrency_kit::storage::values::StorageValue;
use tokio_util::codec::{Decoder, Encoder};

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

impl Encoder<RawMessage> for MsgPacketCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: RawMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = msg.into_bytes();
        let size = msg.len() as u32;
        dst.reserve((size + MSG_SIZE) as usize);
        let mut buf = [0u8; 4];
        BigEndian::write_u32(&mut buf, size);
        dst.extend_from_slice(&buf);
        dst.extend_from_slice(&msg);
        Ok(())
    }
}

