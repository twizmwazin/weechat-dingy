#![allow(dead_code)]
use backtrace::Backtrace;
use byteorder::{ByteOrder, BE};
use libflate::zlib;
use std::clone::Clone;
use std::collections::BTreeMap;
use std::io::{Cursor, Error, Read};

//
// Types
//

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub struct Hdata {
    h_path: Vec<String>,
    keys: Vec<(String, String)>,
    values: Vec<BTreeMap<String, WeechatType>>,
}

impl Hdata {
    // Get value for key at hdata index. Uses WeechatType::unwrap::<T> to return arbitrary types.
    // Returns None if key not found or if unwrap fails
    pub fn get<T>(&self, index: usize, key: &'static str) -> Option<T>
    where
        T: WeechatUnwrappable<T>,
    {
        self.values[index].get(key).and_then(|a| a.unwrap::<T>())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub struct InfoListEntry();

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum WeechatString {
    Null,
    Str(String),
}

impl WeechatString {
    pub fn map<T, F>(self, func: F) -> Option<T>
    where
        F: FnOnce(String) -> T,
    {
        match self {
            WeechatString::Null => None,
            WeechatString::Str(s) => Some(func(s)),
        }
    }

    pub fn to_str(&self) -> String {
        match &self {
            WeechatString::Null => "(null)".to_owned(),
            WeechatString::Str(s) => s.clone(),
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone)]
pub enum WeechatType {
    Char(i8),
    Int(i32),
    Long(i128),
    String(WeechatString),
    Buffer(Option<Vec<u8>>),
    Pointer(u128),
    Time(u128),
    HashTable(BTreeMap<WeechatType, WeechatType>),
    Hdata(Hdata),
    Info(WeechatString, WeechatString),
    InfoList(WeechatString, Vec<(String, WeechatType)>),
    Array(Vec<WeechatType>),
}

// Black magic for allowing trait specialization over primitive types to return
// non-reference types in the type parameter of an Option<T>
pub trait WeechatUnwrappable<T> {
    fn unwrap(wt: &WeechatType) -> Option<T>;
}

// Basic unwrapping just maps enum variants to types and returns Some(val) if you try
// to unwrap to a supported type, or None otherwise.
// Supports variadic matching because of course it does.
macro_rules! basic_unwrappable {
    ($type : ty, $($variant : ident), *) => {
        impl WeechatUnwrappable<$type> for $type {
            fn unwrap(wt: &WeechatType) -> Option<$type> {
                match wt {
                    $(
                        WeechatType::$variant(val) => Some(val.clone()),
                    )*
                    _ => None
                }
            }
        }
    };
}

basic_unwrappable!(i8, Char);
basic_unwrappable!(i32, Int);
basic_unwrappable!(i128, Long);
basic_unwrappable!(u128, Pointer, Time);
basic_unwrappable!(WeechatString, String);
basic_unwrappable!(Option<Vec<u8>>, Buffer);
basic_unwrappable!(Vec<WeechatType>, Array);

// Unwrapping for vectors
impl<T> WeechatUnwrappable<Vec<T>> for Vec<T>
where
    T: WeechatUnwrappable<T>,
{
    fn unwrap(wt: &WeechatType) -> Option<Vec<T>> {
        match wt {
            // Convert to an iter and then map unwrap (into Vec<Option<T>>)
            // Then collect into an Option<Vec<T>> which will be None if any of the Option<T>s are None
            // or Some(Vec<T>()) with the unwrapped contents, otherwise.
            WeechatType::Array(array) => array
                .iter()
                .map(|item| T::unwrap(item))
                .collect::<Option<Vec<T>>>(),
            _ => None,
        }
    }
}

impl<K, V> WeechatUnwrappable<BTreeMap<K, V>> for BTreeMap<K, V>
where
    K: WeechatUnwrappable<K>,
    K: Ord,
    V: WeechatUnwrappable<V>,
{
    fn unwrap(wt: &WeechatType) -> Option<BTreeMap<K, V>> {
        match wt {
            WeechatType::HashTable(map) => {
                let mut new_map: BTreeMap<K, V> = BTreeMap::new();

                for (wt_key, wt_value) in map {
                    let opt_key: Option<K> = wt_key.unwrap::<K>();
                    let opt_value: Option<V> = wt_value.unwrap::<V>();
                    match opt_key {
                        Some(key) => match opt_value {
                            Some(value) => {
                                new_map.insert(key, value);
                            }
                            None => {
                                return None;
                            }
                        },
                        None => {
                            return None;
                        }
                    };
                }

                Some(new_map)
            }
            _ => None,
        }
    }
}

impl WeechatUnwrappable<bool> for bool {
    fn unwrap(wt: &WeechatType) -> Option<bool> {
        wt.unwrap::<i8>().map(|x| x == 1)
    }
}

impl WeechatType {
    // Unwrap into a given type, or None if it cannot be mapped.
    // Supports Vec<T> for arbitrarily nested Vec<>s
    pub fn unwrap<T>(&self) -> Option<T>
    where
        T: WeechatUnwrappable<T>,
    {
        T::unwrap(self)
    }
}

#[derive(Debug)]
pub enum WeechatErrorType {
    IoError,
    UnsupportedType,
    HdataLengthMismatch,
    HdataNullType,
    HdataNullId,
    Other,
}

#[derive(Constructor, Debug)]
pub struct WeechatError {
    pub error: WeechatErrorType,
    pub message: String,
    pub trace: Backtrace,
}

impl std::fmt::Display for WeechatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WeechatError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<Error> for WeechatError {
    fn from(ioError: Error) -> Self {
        WeechatError {
            error: WeechatErrorType::IoError,
            message: format!("{}", ioError),
            trace: Backtrace::new()
        }
    }
}

impl From<WeechatError> for Error {
    fn from(werr: WeechatError) -> Self {
        Error::new(std::io::ErrorKind::InvalidInput, werr)
    }
}

// Reads three-char type signatures into a String
fn parse_type_string(read: &mut Read) -> Result<String, WeechatError> {
    let mut res = String::new();
    let length = read.take(3).read_to_string(&mut res);

    if length.unwrap_or(0) != 3 {
        return Err(WeechatError {
            error: WeechatErrorType::IoError,
            message: "last os error".to_owned(),
            trace: Backtrace::new(),
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

    // val is a binary string
    let ival = i128::from_str_radix(val.as_str(), radix);
    if ival.is_err() {
        return Err(WeechatError {
            error: WeechatErrorType::IoError,
            message: "Int parse error".to_owned(),
            trace: Backtrace::new(),
        });
    }

    Ok(ival.unwrap())
}

// This function will parse all of the types and return a result
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
        "hda" => parse_hda(read),
        "inf" => parse_inf(read),
        "inl" => parse_inl(read),
        "arr" => parse_arr(read),
        _ => Err(WeechatError {
            error: WeechatErrorType::UnsupportedType,
            message: _type,
            trace: Backtrace::new(),
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
    Ok(WeechatType::Buffer(
        parse_str_std(read)?.map(|s| s.into_bytes()),
    ))
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

fn parse_hda(read: &mut Read) -> Result<WeechatType, WeechatError> {
    let h_path = parse_hda_path(read)?;
    let keys = parse_hda_keys(read)?;
    let count = parse_u32(read)?;
    let mut values: Vec<BTreeMap<String, WeechatType>> = Vec::new();

    for _ in 0..count {
        let mut p_path = Vec::new();
        for _ in 0..h_path.len() {
            p_path.push(parse_ptr(read)?);
        }
        let mut vals: BTreeMap<String, WeechatType> = BTreeMap::new();
        for v in &keys {
            vals.insert(v.0.clone(), parse_weechat_type(v.1.clone(), read)?);
        }
        values.push(vals);
    }

    let hda = Hdata {
        h_path,
        keys,
        values,
    };
    Ok(WeechatType::Hdata(hda))
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
        let iname = match parse_str_std(read)? {
            WeechatString::Str(i) => i,
            WeechatString::Null => {
                return Err(WeechatError {
                    error: WeechatErrorType::HdataNullType,
                    message: "".to_owned(),
                    trace: Backtrace::new(),
                });
            }
        };
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

#[derive(Debug)]
struct MessageHeader {
    length: u32,
    compression: u8,
}

impl MessageHeader {
    fn parse(read: &mut Read) -> Result<Option<Self>, WeechatError> {
        let len_buf = &mut [0u8; 4];
        // TODO: error checking?
        read.read_exact(len_buf)?;
        let compression = &mut [0u8; 1];
        read.read_exact(compression)?;
        Ok(Some(MessageHeader {
            length: BE::read_u32(len_buf),
            compression: compression[0],
        }))
    }
}

#[derive(Debug)]
pub struct Message {
    header: MessageHeader,
    pub id: String,
    pub data: Vec<WeechatType>,
}

impl Message {
    pub fn parse(read: &mut Read) -> Result<Option<Message>, WeechatError> {
        MessageHeader::parse(read).and_then(|opt_header| {
            match opt_header {
                Some(header) => {
                    let mut buffer = Vec::new();
                    // TODO: error check this
                    read.take(u64::from(header.length) - 5)
                        .read_to_end(&mut buffer)
                        .unwrap();
                    let decompressed = match header.compression {
                        0 => buffer,
                        1 => {
                            let mut dec = zlib::Decoder::new(buffer.as_slice()).unwrap();
                            let mut dec_buf = Vec::new();
                            dec.read_to_end(&mut dec_buf).unwrap();
                            dec_buf
                        }
                        // TODO: error here
                        _ => buffer,
                    };
                    let mut cursor = Cursor::new(decompressed);

                    let id: String = match parse_str_std(&mut cursor)? {
                        WeechatString::Str(i) => i,
                        WeechatString::Null => "".to_owned(),
                    };
                    let mut data = Vec::new();
                    while let Ok(parse) = parse_type_string(&mut cursor) {
                        data.push(parse_weechat_type(parse, &mut cursor)?);
                    }
                    Ok(Some(Message { header, id, data }))
                },
                None => Ok(None)
            }
        })
    }
}

//
// Helper functions
//
fn handle_io_error() -> WeechatError {
    WeechatError {
        error: WeechatErrorType::IoError,
        message: "".to_owned(),
        trace: Backtrace::new(),
    }
}

fn parse_str_std(read: &mut Read) -> Result<WeechatString, WeechatError> {
    let mut _read_res: Result<(), Error> = Ok(());
    let len = parse_u32(read)?;
    if len == 0xFF_FF_FF_FF {
        // Null string
        return Ok(WeechatString::Null);
    }
    let mut res = String::new();
    let str_read_res = read.take(u64::from(len)).read_to_string(&mut res);
    if str_read_res.is_err() {
        println!("error: {:?}", str_read_res.err());
        for _ in 0..100 {
            println!("{:?}", Backtrace::new());
        }
        return Err(handle_io_error());
    }
    Ok(WeechatString::Str(res))
}

fn parse_u32(read: &mut Read) -> Result<u32, WeechatError> {
    let buf = &mut [0u8; 4];
    match read.read_exact(buf) {
        Ok(_) => Ok(BE::read_u32(buf)),
        _ => Err(handle_io_error()),
    }
}

fn parse_hda_path(read: &mut Read) -> Result<Vec<String>, WeechatError> {
    let base = match parse_str_std(read)? {
        WeechatString::Str(e) => e,
        WeechatString::Null => {
            return Err(WeechatError::new(
                WeechatErrorType::HdataNullId,
                "".into(),
                Backtrace::new(),
            ));
        }
    };
    Ok(base.split('/').map(|s| s.to_string()).collect())
}

fn parse_hda_keys(read: &mut Read) -> Result<Vec<(String, String)>, WeechatError> {
    let keys = match parse_str_std(read)? {
        WeechatString::Str(e) => e,
        WeechatString::Null => {
            return Err(WeechatError::new(
                WeechatErrorType::HdataNullId,
                "".into(),
                Backtrace::new(),
            ));
        }
    };
    let split_keys: Vec<&str> = keys.split(',').collect();
    let mut res: Vec<(String, String)> = Vec::new();
    for i in split_keys {
        let i_split: Vec<&str> = i.split(':').collect();
        res.push((i_split[0].to_string(), i_split[1].to_string()));
    }
    Ok(res)
}
