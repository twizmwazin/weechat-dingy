#![allow(dead_code)]
use std::collections::HashMap;
use std::io::{Error, Read};

use byteorder::{ByteOrder, BE};

//
// Types
//

pub struct Hdata {
    keys: HashMap<String, String>,
    path: String,
    items: Vec<HashMap<String, Box<WeechatType>>>,
}

pub struct InfoListEntry();

pub trait WeechatType {}

pub struct WeechatChar(i8);
pub struct WeechatInt(i32);
pub struct WeechatLong(String);
pub struct WeechatString(String);
pub struct WeechatBuffer(Vec<u8>);
pub struct WeechatPointer(u128);
pub struct WeechatTime(String);
pub struct WeechatHashTable<K: WeechatType, V: WeechatType>(HashMap<K, V>);
pub struct WeechatHdata(Hdata);
pub struct WeechatInfo(String, String);
pub struct WeechatInfoList(String, Vec<(String, Box<WeechatType>)>);
pub struct WeechatArray<T: WeechatType>(Vec<T>);

impl WeechatType for WeechatChar {}
impl WeechatType for WeechatInt {}
impl WeechatType for WeechatLong {}
impl WeechatType for WeechatString {}
impl WeechatType for WeechatBuffer {}
impl WeechatType for WeechatPointer {}
impl WeechatType for WeechatTime {}
impl<K: WeechatType, V: WeechatType> WeechatType for WeechatHashTable<K, V> {}
impl WeechatType for WeechatHdata {}
impl WeechatType for WeechatInfo {}
impl WeechatType for WeechatInfoList {}
impl<T: WeechatType> WeechatType for WeechatArray<T> {}

pub enum WeechatErrorType {
    IoError,
    UnsupportedType,
    Other,
}

pub struct WeechatError {
    pub error: WeechatErrorType,
    pub message: String,
}

// This function will parse all of the types and return a result
// TODO: implementation
fn parse_weechat_type(_type: String, read: &mut Read) -> Result<Box<WeechatType>, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    match _type.as_ref() {
        "chr" => {
            let buf = &mut [0u8; 1];
            _read_res = read.read_exact(buf);
            if _read_res.is_err() {
                return handle_io_error();
            }
            Ok(Box::new(WeechatChar(buf[0] as i8)))
        }
        "int" => {
            let buf = &mut [0u8; 4];
            _read_res = read.read_exact(buf);
            if _read_res.is_err() {
                return handle_io_error();
            }
            Ok(Box::new(WeechatInt(BE::read_i32(buf))))
        }
        "lon" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "str" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "buf" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "ptr" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "tim" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "htb" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "hda" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "inf" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "inl" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "arr" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        _ => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
    }
}

//
// Actual composed Messages
//

struct MessageHeader {
    length: u32,
    compression: u8,
}

struct Message {
    header: MessageHeader,
    id: String,
    data: Vec<Box<WeechatType>>,
}

//
// Helper functions
//
fn handle_io_error() -> Result<Box<WeechatType>, WeechatError> {
    Err(WeechatError {
        error: WeechatErrorType::IoError,
        message: "".to_owned(),
    })
}
