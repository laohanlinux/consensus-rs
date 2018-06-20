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

pub type TId = u64;
pub type TAddr = net::SocketAddr;
pub type TValue = Vec<u8>;
pub type TData = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request<TId:'static, TAddr: 'static, TValue> {
    pub caller: Node<TId, TAddr>,
    pub request_id: TId,
    pub payload: RequestPayload<TId, TValue>,
}

impl <TId:'static, TAddr:'static, TValue> Request <TId, TAddr, TValue>{
    pub fn new(node: Node<TId, TAddr>, request_id: TId, payload: RequestPayload<TId, TValue>)
        -> Request<TId, TAddr, TValue>{
        Request {
            caller: node,
            request_id,
            payload,
        }
    }
}

impl<TId:'static, TAddr:'static, TValue:'static> Message for Request <TId, TAddr, TValue> {
    type Result = Response<TId, TAddr, TValue>;
}

/// Payload in the request.
#[derive(Serialize, Deserialize, Debug, Clone, Message)]
pub enum RequestPayload<TId, TValue> {
    Ping,
    FindNode(TId),
    FindValue(TId),
    Store(TId, TValue)
}

/// Payload in the response.
#[derive(Serialize, Deserialize, Debug, Clone, Message)]
pub enum ResponsePayload<TId, TAddr: 'static, TValue> {
    NodesFound(Vec<Node<TId, TAddr>>),
    ValueFound(TValue),
    NoResult
}

/// Response structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct Response<TId:'static, TAddr:'static, TValue:'static> {
    pub request: Request<TId, TAddr, TValue>,
    pub responder: Node<TId, TAddr>,
    pub payload: ResponsePayload<TId, TAddr, TValue>
}

impl<TId:'static, TAddr:'static, TValue:'static> Response<TId, TAddr, TValue> {
    pub fn new(request: Request<TId, TAddr, TValue>, node: Node<TId, TAddr>, payload: ResponsePayload<TId, TAddr, TValue>) -> Response<TId, TAddr, TValue> {
        Response{
            request,
            responder: node,
            payload,
        }
    }
}

impl <A, M, TId, TAddr, TValue> MessageResponse<A, M> for Response<TId, TAddr, TValue>
    where  A: Actor,
           M:Message<Result = Response<TId, TAddr, TValue>>
{
    fn handle<R: ResponseChannel<M>>(self, _:&mut A::Context, tx: Option<R>) {
       if let Some(tx) = tx {
           tx.send(self);
       }
    }
}

/// |2Byte|xxxx|
/// |msg Size|xxxx|
//pub struct InboundCodec;
pub struct Codec;

impl Decoder for Codec {
    type Item = Request<TId, TAddr, TValue>;
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
            Ok(Some(json::from_slice::<Request<TId, TAddr, TValue>>(&buf)?))
        }else {
            Ok(None)
        }
    }
}

impl Encoder for Codec {
    type Item = Response<TId, TAddr, TValue>;
    type Error = io::Error;
    fn encode(&mut self, msg: Response<TId, TAddr, TValue>,
              dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);
        Ok(())
    }
}

// client
pub struct OutboundCode;

impl Decoder for OutboundCode {
    type Item = Response<TId, TAddr, TValue>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() > 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };
        if src.len() >= size +2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<Response<TId, TAddr, TValue>>(&buf)?))
        }else {
            Ok(None)
        }
    }
}

impl Encoder for OutboundCode {
    type Item = Request<TId, TAddr, TValue>;
    type Error = io::Error;

    fn encode(&mut self, msg: Request<TId, TAddr, TValue>,
              dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);
        Ok(())
    }
}
