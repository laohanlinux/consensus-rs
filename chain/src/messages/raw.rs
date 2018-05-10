
/// Message writer.
#[derive(Debug, PartialEq)]
pub struct MessageWriter {
    raw: Vec<u8>,
}

impl MessageWriter {
    /// Creates a `MessageWriter` instance with given parameters.
    pub fn new(
        protocol_version: u8,
        service_id: u16,
        message_type: u16,
        payload_length: usize,
    ) -> Self {
        // First byte is reserved for backward-compatibility and better alignment.
        let mut raw = MessageWriter {
            raw: vec![0; HEADER_LENGTH + payload_length],
        };
        raw.set_version(protocol_version);
        raw.set_service_id(service_id);
        raw.set_message_type(message_type);
        raw
    }

    /// Sets version.
    fn set_version(&mut self, version: u8) {
        self.raw[1] = version
    }

    /// Sets the service id.
    fn set_service_id(&mut self, service_id: u16) {
        LittleEndian::write_u16(&mut self.raw[4..6], service_id)
    }

    /// Sets the message type.
    fn set_message_type(&mut self, message_type: u16) {
        LittleEndian::write_u16(&mut self.raw[2..4], message_type)
    }

    /// Sets the length of the payload.
    fn set_payload_length(&mut self, length: usize) {
        LittleEndian::write_u32(&mut self.raw[6..10], length as u32)
    }

    /// Writes given field to the given offset.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn write<'a, F: Field<'a>>(&'a mut self, field: F, from: Offset, to: Offset) {
        field.write(
            &mut self.raw,
            from + HEADER_LENGTH as Offset,
            to + HEADER_LENGTH as Offset,
        );
    }

    /// Signs the message with the given secret key.
    pub fn sign(mut self, secret_key: &SecretKey) -> MessageBuffer {
        let payload_length = self.raw.len() + SIGNATURE_LENGTH;
        self.set_payload_length(payload_length);
        let signature = sign(&self.raw, secret_key);
        self.raw.extend_from_slice(signature.as_ref());
        MessageBuffer { raw: self.raw }
    }

    /// Appends the given signature to the message.
    pub fn append_signature(mut self, signature: &Signature) -> MessageBuffer {
        let payload_length = self.raw.len() + SIGNATURE_LENGTH;
        self.set_payload_length(payload_length);
        self.raw.extend_from_slice(signature.as_ref());
        debug_assert_eq!(self.raw.len(), payload_length);
        MessageBuffer { raw: self.raw }
    }
}