use byteorder::{ByteOrder, LittleEndian};

use std::{convert, fmt::Debug, ops::Deref, sync};

/// Length of the message header.
pub const HEADER_LENGTH: usize = 10;
/// Version of the protocol. Different versions are incompatible.
pub const PROTOCOL_MAJOR_VERSION: u8 = 0;

pub struct RawMessage(sync::Arc<MessageBuffer>);

impl RawMessage {
    pub fn new(buffer: MessageBuffer) -> Self {RawMessage(sync::Arc::new(buffer))}

    pub fn from_vec(vec: Vec<u8>) ->Self {
        RawMessage(sync::Arc::new(MessageBuffer::from_vec(vec)))
    }
}

impl Deref for RawMessage {
    type Target = MessageBuffer;

    fn deref(&self) ->&self::Target{&self.0}
}

impl AsRef<[u8]> for RawMessage {
    fn as_ref(&self) -> &[u8] {self.0.as_ref().as_ref()}
}

/// |version(1byte)|msg_type(2byte)|payload_length(4byte)|body|

/// A raw message represented by the bytes buffer.
#[derive(Debug, PartialEq)]
pub struct MessageBuffer {
    raw: Vec<u8>,
}

impl MessageBuffer {
    pub fn from_vec(raw: Vec<u8>) ->MessageBuffer {
        MessageBuffer{raw}
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn version(&self) -> u8 {
        self.raw[1]
    }

    pub fn message_type(&self) -> u16 {
        LittleEndian::read_u16(&self.raw[2..4])
    }
}

impl convert::AsRef<[u8]> for MessageBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

/// Message writer
pub struct MessageWriter{
    raw: Vec<u8>,
}

impl MessageWriter {
    pub fn new(
        protocol_version: u8,
        message_type: u16,
        payload_length: usize,
    ) ->Self {
        let mut raw = MessageWriter {
            raw: vec![0; HEADER_LENGTH + payload_length],
        };

    }

    /// sets version
    fn set_version(&mut self, version: u8) {
        self.raw[1] = version;
    }

    /// sets the message type
    fn set_message_type(&mut self, message_type: u16) {
        LittleEndian::write_u16(&mut self.raw[2..4], message_type);
    }


    /// set the length of the payload
    fn set_payload_length(&mut self, length: usize) {
        LittleEndian::write_u32(&mut self.raw[5..9], length as u32);
    }


    // TODO
    pub fn sign() {}

    // TODO
    pub fn append_signature() {}
}