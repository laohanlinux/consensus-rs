//! Automatically generated rust module for 'block-dpos.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Block {
    pub height: u64,
    pub timestamp: i64,
    pub generator: Vec<i32>,
}

impl<'a> MessageRead<'a> for Block {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Block {
            height: 8u64,
            timestamp: 8i64,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.height = r.read_uint64(bytes)?,
                Ok(16) => msg.timestamp = r.read_int64(bytes)?,
                Ok(26) => msg.generator = r.read_packed(bytes, |r, bytes| r.read_int32(bytes))?,
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
        + if self.timestamp == 8i64 { 0 } else { 1 + sizeof_varint(*(&self.timestamp) as u64) }
        + if self.generator.is_empty() { 0 } else { 1 + sizeof_len(self.generator.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.height != 8u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.height))?; }
        if self.timestamp != 8i64 { w.write_with_tag(16, |w| w.write_int64(*&self.timestamp))?; }
        w.write_packed_with_tag(26, &self.generator, |w, m| w.write_int32(*m), &|m| sizeof_varint(*(m) as u64))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Transaction<'a> {
    pub amount: u64,
    pub from: Vec<u32>,
    pub data: Option<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Transaction<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Transaction {
            amount: 8u64,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.amount = r.read_uint64(bytes)?,
                Ok(18) => msg.from = r.read_packed(bytes, |r, bytes| r.read_uint32(bytes))?,
                Ok(26) => msg.data = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Transaction<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.amount == 8u64 { 0 } else { 1 + sizeof_varint(*(&self.amount) as u64) }
        + if self.from.is_empty() { 0 } else { 1 + sizeof_len(self.from.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
        + self.data.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.amount != 8u64 { w.write_with_tag(8, |w| w.write_uint64(*&self.amount))?; }
        w.write_packed_with_tag(18, &self.from, |w, m| w.write_uint32(*m), &|m| sizeof_varint(*(m) as u64))?;
        if let Some(ref s) = self.data { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

