use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use tokio_io::codec::{Decoder, Encoder};
use kad::base::Node;
use std::net;
use std::io;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug, Message)]
pub struct Request<TId, TAddr, TValue>{
    pub caller: Node<TId, TAddr>,
    pub request_id: u64,
    pub payload: RequestPayload<u64, TValue>,
}

/// Payload in the request.
#[derive(Serialize, Deserialize, Debug, Message)]
pub enum RequestPayload<GenericId, TValue> {
    Ping,
    FindNode(GenericId),
    FindValue(GenericId),
    Store(GenericId, TValue)
}

/// Payload in the response.
#[derive(Serialize, Deserialize, Debug, Message)]
pub enum ResponsePayload<TId, TAddr, TValue> {
    NodesFound(Vec<Node<TId, TAddr>>),
    ValueFound(TValue),
    NoResult
}

/// Response structure.
#[derive(Serialize, Deserialize, Debug, Message)]
pub struct Response<TId, TAddr, TValue> {
    pub request: Request<TId, TAddr, TValue>,
    pub responder: Node<TId, TAddr>,
    pub payload: ResponsePayload<TId, TAddr, TValue>
}

/// |2Byte|xxxx|
/// |msg Size|xxxx|
//pub struct InboundCodec;
pub struct Codec;

impl Decoder for Codec {
    type Item = Request<u64, net::SocketAddr, Vec<u8>>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };
        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<Request<u64, net::SocketAddr, Vec<u8>>>(&buf)?))
        }else {
            Ok(None)
        }
    }
}

impl Encoder for Codec {
    type Item = Response<u64, net::SocketAddr, Vec<u8>>;
    type Error = io::Error;
    fn encode(&mut self, msg: Response<u64, net::SocketAddr, Vec<u8>>,
              dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);
        Ok(())
    }
}
//
//pub struct OutboundCode;
//
//impl Decoder for OutboundCode {
//    type Item = Response<u64, net::SocketAddr, Vec<u8>>;
//    type Error = io::Error;
//
//    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//        let size = {
//            if src.len() > 2 {
//                return Ok(None);
//            }
//            BigEndian::read_u16(src.as_ref()) as usize
//        };
//        if src.len() >= size +2 {
//            src.split_to(2);
//            let buf = src.split_to(size);
//            Ok(Some(json::from_slice::<Response<u64, net::SocketAddr, Vec<u8>>>(&buf)?))
//        }else {
//            Ok(None)
//        }
//    }
//}
//
//impl Encoder for OutboundCode {
//    type Item = Request<u64, net::SocketAddr, Vec<u8>>;
//    type Error = io::Error;
//
//    fn encode(
//        &mut self, msg: Request<u64, >
//    )
//}
