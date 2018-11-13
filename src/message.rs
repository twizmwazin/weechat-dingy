use std::collections::HashMap;
use std::io::*;
use std::option::Option;
use std::result::Result;
use std::str;
use std::iter;
use std::vec::Vec;
use byteorder::{BigEndian, ReadBytesExt};

pub enum RelayObjectType {
    SignedChar {
        value: i8,
    },
    SignedInteger {
        value: i32,
    },
    SignedLongInteger {
        value: &'static str,
    },
    String {
        value: Option<&'static str>,
    },
    Bytes {
        value: Option<&'static [u8]>,
    },
    Pointer {
        value: u64,
    },
    Time {
        value: &'static str,
    },
    Hashtable {
        value: HashMap<RelayObjectType, RelayObjectType>,
    },
    HdataContent {
        hpath: &'static str
        /* todo: this struct sucks */
    },
    Info {
        name: Option<&'static str>,
        value: Option<&'static str>,
    },
    Infolist {
        name: Option<&'static str>
        // items: Vec<,
    },
    Array {
        items: Vec<RelayObjectType>
    },
}

pub struct RelayMessageHeader {
    length: u32,
    compression: bool,
    id_int: Option<u32>,
    id: Option<&'static str>,
}

pub struct RelayMessage {
    header: RelayMessageHeader,
    objects: Vec<RelayObjectType>
}

fn eat_u32_be(array: &mut Vec<u8>) -> u32 {
    Cursor::new(&(array.splice(..4, iter::empty::<u8>()).collect::<Vec<u8>>()[0..4])).read_u32::<BigEndian>().unwrap()
}

fn eat_u8(array: &mut Vec<u8>) -> u8 {
    array.splice(..1, iter::empty::<u8>()).collect::<Vec<u8>>()[0]
}

fn eat_string(array: &mut Vec<u8>) -> Result<Option<String>, &'static str> {
    let length = eat_u32_be(array);

    if length == 0xFFFFFFFF {
        return Ok(Option::None);
    }
    let bytes = array.splice(..(length as usize), iter::empty::<u8>()).collect::<Vec<u8>>();
    Ok(Option::Some(String::from_utf8(bytes).unwrap()))
}

fn from_u32_be(value: u32) -> [u8; 4] {
    [
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        (value) as u8,
    ]
}

impl RelayMessage {
    pub fn from_bytes(data: &[u8]) -> Result<RelayMessage, &'static str> {
        let mut eating : Vec<u8> = Vec::from(data);

        let msg_len: u32 = eat_u32_be(&mut eating);
        let compression = eat_u8(&mut eating);
        if compression != 0 {
            return Err("Not supporting compression yet");
        }

        let header = RelayMessageHeader {
            length: msg_len,
            compression: compression != 0,
            id_int: None,
            id: None
        };

        let objects : Vec<RelayObjectType> = Vec::default();

        let mut res : RelayMessage = RelayMessage {
            header: header,
            objects: objects
        };
        println!("Msg len: {}", res.header.length);

        Ok(res)
    }
}
