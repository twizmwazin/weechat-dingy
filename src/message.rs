#![allow(dead_code)]
use libflate::zlib::Decoder;
use std::collections::BTreeMap;
use std::io::{Cursor, Error, Read};

use byteorder::{ByteOrder, BE};

//
// Types
//

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct Hdata {
    keys: BTreeMap<String, String>,
    path: String,
    items: Vec<BTreeMap<String, WeechatType>>,
}

pub struct InfoListEntry();

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum WeechatType {
    Char(i8),
    Int(i32),
    Long(i128),
    String(String),
    Buffer(Vec<u8>),
    Pointer(u128),
    Time(u128),
    HashTable(BTreeMap<WeechatType, WeechatType>),
    Hdata(Hdata),
    Info(String, String),
    InfoList(String, Vec<(String, WeechatType)>),
    Array(Vec<WeechatType>),
}

pub enum WeechatErrorType {
    IoError,
    UnsupportedType,
    Other,
}

pub struct WeechatError {
    pub error: WeechatErrorType,
    pub message: String,
}

// Reads three-char type signatures into a String
fn parse_type_string(read: &mut Read) -> Result<String, WeechatError> {
    let mut res = String::new();
    let length = read.take(3).read_to_string(&mut res);

    if length.unwrap_or(0) != 3 {
        return Err(WeechatError {
            error: WeechatErrorType::IoError,
            message: "last os error".to_owned(),
        });
    }

    Ok(res)
}

fn parse_str_int(read: &mut Read, radix: u32) -> Result<i128, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    let buf = &mut [0u8; 1];
    _read_res = read.read_exact(buf);
    if _read_res.is_err() {
        return Err(handle_io_error());
    }
    let len = buf[0] as u8;
    let mut val = String::new();
    let val_res = read.take(u64::from(len)).read_to_string(&mut val);

    if val_res.is_err() {
        return Err(handle_io_error());
    }

    //val is a binary string
    let ival = i128::from_str_radix(val.as_str(), radix);
    if ival.is_err() {
        return Err(WeechatError {
            error: WeechatErrorType::IoError,
            message: "Int parse error".to_owned(),
        });
    }

    Ok(ival.unwrap())
}

// This function will parse all of the types and return a result
// TODO: implementation
fn parse_weechat_type(_type: String, read: &mut Read) -> Result<WeechatType, WeechatError> {
    match _type.as_ref() {
        "chr" => parse_chr(read),
        "int" => parse_int(read),
        "lon" => parse_lon(read),
        "str" => parse_str(read),
        "buf" => parse_buf(read),
        "ptr" => parse_ptr(read),
        "tim" => parse_tim(read),
        "htb" => parse_htb(read),
        "hda" => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
        "inf" => parse_inf(read),
        "inl" => parse_inl(read),
        "arr" => parse_arr(read),
        _ => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
        }),
    }
}

fn parse_chr(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    let buf = &mut [0u8; 1];
    _read_res = read.read_exact(buf);
    if _read_res.is_err() {
        return Err(handle_io_error());
    }
    Ok(WeechatType::Char(buf[0] as i8))
}

fn parse_int(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    let buf = &mut [0u8; 4];
    _read_res = read.read_exact(buf);
    if _read_res.is_err() {
        return Err(handle_io_error());
    }
    Ok(WeechatType::Int(BE::read_i32(buf)))
}

fn parse_lon(read: &mut Read) -> Result<WeechatType, WeechatError> {
    Ok(WeechatType::Int(parse_str_int(read, 10)? as i32))
}

fn parse_str(read: &mut Read) -> Result<WeechatType, WeechatError> {
    Ok(WeechatType::String(parse_str_std(read)?))
}

fn parse_buf(read: &mut Read) -> Result<WeechatType, WeechatError> {
    Ok(WeechatType::Buffer(parse_str_std(read)?.into_bytes()))
}

fn parse_ptr(read: &mut Read) -> Result<WeechatType, WeechatError> {
    Ok(WeechatType::Pointer(parse_str_int(read, 16)? as u128))
}

fn parse_tim(read: &mut Read) -> Result<WeechatType, WeechatError> {
    Ok(WeechatType::Time(parse_str_int(read, 10)? as u128))
}

fn parse_htb(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let key_type = parse_type_string(read)?;
    let val_type = parse_type_string(read)?;
    let count = parse_u32(read)?;
    let mut htb: BTreeMap<WeechatType, WeechatType> = BTreeMap::new();
    for _ in 0..count {
        htb.insert(
            parse_weechat_type(key_type.clone(), read)?,
            parse_weechat_type(val_type.clone(), read)?,
        );
    }
    Ok(WeechatType::HashTable(htb))
}

fn parse_inf(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let name = match parse_str_std(read) {
        Ok(n) => n,
        Err(_) => return Err(handle_io_error()),
    };
    let value = match parse_str_std(read) {
        Ok(n) => n,
        Err(_) => return Err(handle_io_error()),
    };
    Ok(WeechatType::Info(name, value))
}

fn parse_inl(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let name = parse_str_std(read)?;
    let count = parse_u32(read)?;
    let mut items = Vec::new();
    for _ in 0..count {
        let iname = parse_str_std(read)?;
        let _type = parse_type_string(read)?;
        let obj = parse_weechat_type(_type, read)?;
        items.push((iname, obj));
    }
    Ok(WeechatType::InfoList(name, items))
}

fn parse_arr(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let _type = parse_type_string(read)?;
    let len = parse_u32(read)?;
    let mut res: Vec<WeechatType> = Vec::new();
    for _ in 0..len {
        res.push(parse_weechat_type(_type.clone(), read)?);
    }
    Ok(WeechatType::Array(res))
}

//
// Actual composed Messages
//

struct MessageHeader {
    length: u32,
    compression: u8,
}

impl MessageHeader {
    fn parse(read: &mut Read) -> Self {
        let len_buf = &mut [0u8; 4];
        // TODO: error checking?
        read.read_exact(len_buf).unwrap();
        let compression = &mut [0u8; 1];
        read.read_exact(compression).unwrap();
        MessageHeader {
            length: BE::read_u32(len_buf),
            compression: compression[0],
        }
    }
}

pub struct Message {
    header: MessageHeader,
    id: String,
    pub data: Vec<WeechatType>,
}

impl Message {
    pub fn parse(read: &mut Read) -> Result<Message, WeechatError> {
        let header = MessageHeader::parse(read);
        let mut buffer = Vec::new();
        // TODO: error check this
        read.take(u64::from(header.length) - 5)
            .read_to_end(&mut buffer)
            .unwrap();
        let decompressed = match header.compression {
            0 => buffer,
            1 => {
                let mut dec = Decoder::new(buffer.as_slice()).unwrap();
                let mut dec_buf = Vec::new();
                dec.read_to_end(&mut dec_buf).unwrap();
                dec_buf
            }
            // TODO: error here
            _ => buffer,
        };
        let mut cursor = Cursor::new(decompressed);
        let id: String = parse_str_std(&mut cursor)?;
        let mut data = Vec::new();
        while let Ok(parse) = parse_type_string(&mut cursor) {
            data.push(parse_weechat_type(parse, &mut cursor)?);
        }
        Ok(Message { header, id, data })
    }
}

//
// Helper functions
//
fn handle_io_error() -> WeechatError {
    WeechatError {
        error: WeechatErrorType::IoError,
        message: "".to_owned(),
    }
}

fn parse_str_std(read: &mut Read) -> Result<String, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    let len = parse_u32(read)?;
    if len == 0xFF_FF_FF_FF {
        //Empty string
        return Ok(String::new());
    }
    let mut res = String::new();
    let str_read_res = read.take(u64::from(len)).read_to_string(&mut res);
    if str_read_res.is_err() {
        return Err(handle_io_error());
    }
    Ok(res)
}

fn parse_u32(read: &mut Read) -> Result<u32, WeechatError> {
    let buf = &mut [0u8; 4];
    match read.read_exact(buf) {
        Ok(_) => Ok(BE::read_u32(buf)),
        _ => Err(handle_io_error()),
    }
}
