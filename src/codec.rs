use std::io::Cursor;
use bytes::BufMut;
use bytes::BytesMut;
use tokio::codec::{Decoder, Encoder};
use tokio::prelude::*;
use crate::command::Command;
use crate::message::Message;

pub struct WeechatCodec;

impl WeechatCodec {
    pub fn new() -> WeechatCodec {
        WeechatCodec{}
    }
}

impl Decoder for WeechatCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 5 {
            return Ok(None);
        }
        let mut cursor = Cursor::new(&src);
        let parsed = Message::parse(&mut cursor);
        src.split_to(cursor.position() as _);
        
        parsed.map_err(|werr| Self::Error::from(werr))
    }
}

impl Encoder for WeechatCodec {
    type Item = Box<Command+Send>;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut value : Vec<u8> = vec!();
        let size = item.encode(&mut value)?;
        Ok(dst.put(value))
    }
}
