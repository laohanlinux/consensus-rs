use actix::prelude::*;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use tokio_io::codec::{Decoder, Encoder};
use actix::prelude::*;
use actix::dev::{MessageResponse, ResponseChannel};
use kad::base::Node;
use std::net;
use std::io;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct MessagesCodec{
    max_message_len: u32,
    // TODO add NoiseWrapper
}

impl MessagesCodec {
    pub fn new(max_message_len: u32) -> MessagesCodec {
        MessagesCodec{
            max_message_len: max_message_len,
        }
    }
}
//
//impl Decoder for MessagesCodec {
//
//}