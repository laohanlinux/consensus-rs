//! Automatically generated rust module for 'block-dpos.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Block {
    pub height: u64,
    pub timestamp: u64,
}

impl<'a> MessageRead<'a> for Block {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Block {
            height: 8u64,
            timestamp: 8u64,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.height = r.read_uint64(bytes)?,
                Ok(16) => msg.timestamp = r.read_uint64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Block {
    fn get_size(&self) -> usize {
        0
        + if self.height == 8u64 { 0 } else { 1 + sizeof_varint(*(&self.height) as u64) }
        + if self.timestamp == 8u64 { 0 } else { 1 + sizeof_varint(*(&self.timestamp) as u64) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.height != 8u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.height))?; }
        if self.timestamp != 8u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.timestamp))?; }
        Ok(())
    }
}

