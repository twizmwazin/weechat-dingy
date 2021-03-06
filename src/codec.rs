use libdingy::command::Command;
use libdingy::message::Message;
use libdingy::message::MessageHeader;
use bytes::BufMut;
use bytes::BytesMut;
use std::io::Cursor;
use tokio::codec::{Decoder, Encoder};

pub struct WeechatCodec;

impl WeechatCodec {
    pub fn new() -> WeechatCodec {
        WeechatCodec {}
    }
}

impl Decoder for WeechatCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 5 {
            return Ok(None);
        }
        let mut header_cursor = Cursor::new(&src);
        let opt_header = MessageHeader::parse(&mut header_cursor)?;

        if let Some(header) = opt_header {
            if src.len() < header.length as usize {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        println!("{:?}", src.as_mut());

        let mut cursor = Cursor::new(&src);
        let parsed = Message::parse(&mut cursor);
        src.split_to(cursor.position() as _);

        parsed.map_err(|werr| Self::Error::from(werr))
    }
}

impl Encoder for WeechatCodec {
    type Item = Box<Command + Send>;
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let mut value: Vec<u8> = vec![];
        let _size = item.encode(&mut value)?;
        Ok(dst.put(value))
    }
}
