use crate::command::*;
use crate::message::*;
use crate::sync::*;
use std::io::Cursor;
use std::ptr::{null, null_mut};
use std::slice;

//-----------------------------------------------------------------------------

fn test_ptr<T>(p: *const T) -> Option<*const T> {
    if p.is_null() {
        None
    } else {
        Some(p)
    }
}

unsafe fn str_from_raw(bytes: *const u8, length: usize) -> Option<String> {
    test_ptr(bytes).and_then(|bytes| {
        let arr = slice::from_raw_parts(bytes, length);
        match String::from_utf8(Vec::from(arr)) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    })
}

unsafe fn str_to_raw(s: Option<Vec<u8>>, buffer: *mut u8, buffer_length: usize) -> usize {
    s.map_or(0, move |s| {
        let copy_length = buffer_length.min(s.len());
        let buf = slice::from_raw_parts_mut(buffer, buffer_length);
        buf[..copy_length].copy_from_slice(&s[..copy_length]);
        copy_length
    })
}

unsafe fn count_to_opt_hcl(count: *const i32) -> Option<HdataCommandLength> {
    test_ptr(count).map(|count| match *count {
        0 => HdataCommandLength::Infinite,
        count => HdataCommandLength::Finite(count)
    })
}

/// Create an init command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param password: Password for server or null
/// @param password_length: Length of password string
/// @param compression: Compression type or null
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_init_print(id: *const u8, id_length: usize, password: *const u8, password_length: usize, compression: *const CompressionType, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let password = str_from_raw(password, password_length);
    let compression = compression.as_ref().cloned();

    // Print command
    let command = InitCommand::new(id, password, compression);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            // Copy out
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create an hdata command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param hdata: Name of hdata
/// @param hdata_length: Length of hdata string
/// @param pointer: Pointer to infolist or null
/// @param pointer_length: Length of pointer string
/// @param pointer_count: Count for pointer parameter
/// @param var_names: List of strings or null
/// @param var_counts: List of counts or null
/// @param var_names_lengths: List of lengths of vars or null
/// @param vars_length: Number of vars
/// @param keys: List of strings or null
/// @param keys_lengths: List of lengths of keys or null
/// @param keys_length: Number of keys
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_hdata_print(id: *const u8, id_length: usize, hdata: *const u8, hdata_length: usize, pointer: *const u8, pointer_length: usize, pointer_count: *const i32, var_names: *const *const u8, var_counts: *const *const i32, var_names_lengths: *const usize, vars_length: usize, keys: *const *const u8, keys_lengths: *const usize, keys_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let hdata = str_from_raw(hdata, hdata_length);
    let pointer = str_from_raw(pointer, pointer_length);
    let pointer_count = count_to_opt_hcl(pointer_count);

    let vars = match (test_ptr(var_names), test_ptr(var_counts), test_ptr(var_names_lengths)) {
        (Some(var_names), Some(var_counts), Some(var_names_lengths)) => {
            let s = slice::from_raw_parts(var_names, vars_length);
            let o = slice::from_raw_parts(var_counts, vars_length);
            let lengths = slice::from_raw_parts(var_names_lengths, vars_length);

            s.into_iter().zip(lengths.into_iter()).zip(o.into_iter()).map(|((&arg, &length), &count)| {
                let count = count_to_opt_hcl(count);
                str_from_raw(arg, length).map(|s| (s, count))
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    }.into_iter().flatten().collect::<Vec<_>>();

    let keys = match (test_ptr(keys), test_ptr(keys_lengths)) {
        (Some(keys), Some(keys_lengths)) => {
            let s = slice::from_raw_parts(keys, keys_length);
            let lengths = slice::from_raw_parts(keys_lengths, keys_length);

            s.into_iter().zip(lengths.into_iter()).map(|(&arg, &length)| {
                str_from_raw(arg, length)
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    };

    match (hdata, pointer) {
        (Some(hdata), Some(pointer)) => {
            let command = HdataCommand::new(id, hdata, (pointer, pointer_count), vars, keys);
            let mut rbuf: Vec<u8> = vec![];
            match command.encode(&mut Cursor::new(&mut rbuf)) {
                Ok(_) => {
                    str_to_raw(Some(rbuf), output, output_length)
                },
                Err(_) => 0
            }
        },
        _ => 0
    }
}

/// Create an info command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param name: Info name
/// @param name_length: Length of name string
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_info_print(id: *const u8, id_length: usize, name: *const u8, name_length: usize, output: *mut u8, output_length: usize) -> usize {
    let id = str_from_raw(id, id_length);
    let name = str_from_raw(name, name_length);

    match name {
        Some(name) => {
            let command = InfoCommand::new(id, name);
            let mut rbuf: Vec<u8> = vec![];
            match command.encode(&mut Cursor::new(&mut rbuf)) {
                Ok(_) => {
                    str_to_raw(Some(rbuf), output, output_length)
                },
                Err(_) => 0
            }
        },
        None => 0
    }
}

/// Create an infolist command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param name: Name of infolist
/// @param name_length: Length of name string
/// @param pointer: Pointer to infolist or null
/// @param pointer_length: Length of pointer string
/// @param arguments: List of strings or null
/// @param arguments_lengths: List of lengths of arguments or null
/// @param arguments_length: Number of arguments
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_infolist_print(id: *const u8, id_length: usize, name: *const u8, name_length: usize, pointer: *const u8, pointer_length: usize, arguments: *const *const u8, arguments_lengths: *const usize, arguments_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let name = str_from_raw(name, name_length);
    let pointer = str_from_raw(pointer, pointer_length);
    let arguments = match (test_ptr(arguments), test_ptr(arguments_lengths)) {
        (Some(arguments), Some(arguments_lengths)) => {
            let s = slice::from_raw_parts(arguments, arguments_length);
            let lengths = slice::from_raw_parts(arguments_lengths, arguments_length);

            s.into_iter().zip(lengths.into_iter()).map(|(&arg, &length)| {
                str_from_raw(arg, length)
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    };

    match name {
        Some(name) => {
            let command = InfoListCommand::new(id, name, pointer, arguments);
            let mut rbuf: Vec<u8> = vec![];
            match command.encode(&mut Cursor::new(&mut rbuf)) {
                Ok(_) => {
                    str_to_raw(Some(rbuf), output, output_length)
                },
                Err(_) => 0
            }
        },
        None => 0
    }
}

/// Create a nicklist command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param buffer: Buffer for list or null
/// @param buffer_length: Length of buffer string
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_nicklist_print(id: *const u8, id_length: usize, buffer: *const u8, buffer_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let buffer = str_from_raw(buffer, buffer_length);

    // Print command
    let command = NicklistCommand::new(id, buffer);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            // Copy out
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create an input command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param buffer: Buffer for input or null
/// @param buffer_length: Length of buffer string
/// @param data: Buffer for data or null
/// @param data_length: Length of data string
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_input_print(id: *const u8, id_length: usize, buffer: *const u8, buffer_length: usize, data: *const u8, data_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let buffer = str_from_raw(buffer, buffer_length);
    let data = str_from_raw(data, data_length);

    match (buffer, data) {
        (Some(buffer), Some(data)) => {
            // Print command
            let command = InputCommand::new(id, buffer, data);
            let mut rbuf: Vec<u8> = vec![];
            match command.encode(&mut Cursor::new(&mut rbuf)) {
                Ok(_) => {
                    // Copy out
                    str_to_raw(Some(rbuf), output, output_length)
                },
                Err(_) => 0
            }
        },
        _ => 0
    }
}

/// Create a sync command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param argument_buffers: List of strings or null
/// @param argument_options: List of options or null
/// @param argument_buffers_lengths: List of lengths of arguments or null
/// @param arguments_length: Number of arguments
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_sync_print(id: *const u8, id_length: usize, argument_buffers: *const *const u8, argument_options: *const SyncOption, argument_buffers_lengths: *const usize, arguments_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let arguments = match (test_ptr(argument_buffers), test_ptr(argument_options), test_ptr(argument_buffers_lengths)) {
        (Some(argument_buffers), Some(argument_options), Some(argument_buffers_lengths)) => {
            let s = slice::from_raw_parts(argument_buffers, arguments_length);
            let o = slice::from_raw_parts(argument_options, arguments_length);
            let lengths = slice::from_raw_parts(argument_buffers_lengths, arguments_length);

            s.into_iter().zip(lengths.into_iter()).zip(o.into_iter()).map(|((&arg, &length), &option)| {
                str_from_raw(arg, length).map(|s| (s, option))
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    }.into_iter().flatten().collect::<Vec<_>>();

    let command = SyncCommand::new(id, arguments);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create a desync command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param argument_buffers: List of strings or null
/// @param argument_options: List of options or null
/// @param argument_buffers_lengths: List of lengths of arguments or null
/// @param arguments_length: Number of arguments
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_desync_print(id: *const u8, id_length: usize, argument_buffers: *const *const u8, argument_options: *const SyncOption, argument_buffers_lengths: *const usize, arguments_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let arguments = match (test_ptr(argument_buffers), test_ptr(argument_options), test_ptr(argument_buffers_lengths)) {
        (Some(argument_buffers), Some(argument_options), Some(argument_buffers_lengths)) => {
            let s = slice::from_raw_parts(argument_buffers, arguments_length);
            let o = slice::from_raw_parts(argument_options, arguments_length);
            let lengths = slice::from_raw_parts(argument_buffers_lengths, arguments_length);

            s.into_iter().zip(lengths.into_iter()).zip(o.into_iter()).map(|((&arg, &length), &option)| {
                str_from_raw(arg, length).map(|s| (s, option))
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    }.into_iter().flatten().collect::<Vec<_>>();

    let command = DesyncCommand::new(id, arguments);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create a test command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_test_print(id: *const u8, id_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);

    let command = TestCommand::new(id);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create a ping command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param arguments: List of strings or null
/// @param arguments_lengths: List of lengths of arguments or null
/// @param arguments_length: Number of arguments
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_ping_print(id: *const u8, id_length: usize, arguments: *const *const u8, arguments_lengths: *const usize, arguments_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);
    let arguments = match (test_ptr(arguments), test_ptr(arguments_lengths)) {
        (Some(arguments), Some(arguments_lengths)) => {
            let s = slice::from_raw_parts(arguments, arguments_length);
            let lengths = slice::from_raw_parts(arguments_lengths, arguments_length);

            s.into_iter().zip(lengths.into_iter()).map(|(&arg, &length)| {
                str_from_raw(arg, length)
            }).collect::<Option<Vec<_>>>()
        },
        _ => None
    };

    let command = PingCommand::new(id, arguments);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

/// Create a quit command
/// @param id: Id of command or null
/// @param id_length: Length of id string
/// @param output: Output buffer
/// @param output_length: Capacity of output buffer
/// @return Number of bytes in full message (even if truncated)
#[no_mangle]
pub unsafe extern "C" fn command_quit_print(id: *const u8, id_length: usize, output: *mut u8, output_length: usize) -> usize {
    // Parameters
    let id = str_from_raw(id, id_length);

    let command = QuitCommand::new(id);
    let mut rbuf: Vec<u8> = vec![];
    match command.encode(&mut Cursor::new(&mut rbuf)) {
        Ok(_) => {
            str_to_raw(Some(rbuf), output, output_length)
        },
        Err(_) => 0
    }
}

//-----------------------------------------------------------------------------

/// Parse a message header
/// @param bytes: List of bytes for message
/// @param length: Length of bytes
/// @return Full length of expected message data
#[no_mangle]
pub unsafe extern "C" fn message_parse_length(bytes: *const u8, length: usize) -> usize {
    let src = slice::from_raw_parts(bytes, length);
    let mut header_cursor = Cursor::new(&src);
    let opt_header = MessageHeader::parse(&mut header_cursor);

    match opt_header {
        Ok(Some(header)) => {
            header.length as usize
        },
        _ => 0
    }
}

/// Parse a message from bytes
/// @param bytes: List of bytes for message
/// @param length: Length of bytes
/// @param parse_length: Length of parsed data
/// @return Message structure pointer, or null
#[no_mangle]
pub unsafe extern "C" fn message_parse(bytes: *const u8, length: usize, parse_length: *mut usize) -> *mut Message {
    let src = slice::from_raw_parts(bytes, length);
    let mut header_cursor = Cursor::new(&src);
    let opt_header = MessageHeader::parse(&mut header_cursor);

    match opt_header {
        Ok(Some(header)) => {
            if src.len() < header.length as usize {
                *parse_length = 0;
                null_mut::<Message>()
            } else {
                let mut cursor = Cursor::new(&src);
                let parsed = Message::parse(&mut cursor);
                // src.split_to(cursor.position() as _);

                match parsed {
                    Ok(Some(msg)) => {
                        let m = Box::from(msg);
                        *parse_length = cursor.position() as usize;
                        Box::leak(m)
                    },
                    _ => {
                        *parse_length = 0;
                        null_mut::<Message>()
                    }
                }

            }
        },
        _ => {
            *parse_length = 0;
            null_mut::<Message>()
        }
    }
}

/// Free a message and all associated data structures
/// @param message: Message to free
#[no_mangle]
pub unsafe extern "C" fn message_free(message: *mut Message) {
    drop(Box::from_raw(message));
}

/// Get id of message
/// @param message: Message
/// @param length: Length of id buffer
/// @return Buffer with id (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn message_id(message: *const Message, length: *mut usize) -> *const u8 {
    *length = (*message).id.len();
    (*message).id.as_ptr()
}

/// Get number of data items in message
/// @param message: Message
/// @return Number of data items
#[no_mangle]
pub unsafe extern "C" fn message_data_count(message: *mut Message) -> usize {
    (*message).data.len()
}

/// Get message's data item at index
/// @param message: Message
/// @param index: Index of data item
/// @return Pointer to data item, or null
#[no_mangle]
pub unsafe extern "C" fn message_data_item(message: *mut Message, index: usize) -> *mut WeechatType {
    match (*message).data.get_mut(index) {
        Some(i) => {
            i as *mut WeechatType
        }
        _ => null_mut()
    }
}

//-----------------------------------------------------------------------------

#[repr(C)]
pub enum WeechatTypeEnum {
    CharType,
    IntType,
    LongType,
    StringType,
    BufferType,
    PointerType,
    TimeType,
    HashTableType,
    HdataType,
    InfoType,
    InfoListType,
    ArrayType,
}

/// Get enum type of WeechatType pointer
/// @param weechat_type: WeechatType pointer
/// @return Which enum type it is
#[no_mangle]
pub unsafe extern "C" fn weechat_type_enum(weechat_type: *const WeechatType) -> WeechatTypeEnum {
    match *weechat_type {
        WeechatType::Char(_) => WeechatTypeEnum::CharType,
        WeechatType::Int(_) => WeechatTypeEnum::IntType,
        WeechatType::Long(_) => WeechatTypeEnum::LongType,
        WeechatType::String(_) => WeechatTypeEnum::StringType,
        WeechatType::Buffer(_) => WeechatTypeEnum::BufferType,
        WeechatType::Pointer(_) => WeechatTypeEnum::PointerType,
        WeechatType::Time(_) => WeechatTypeEnum::TimeType,
        WeechatType::HashTable(_) => WeechatTypeEnum::HashTableType,
        WeechatType::Hdata(_) => WeechatTypeEnum::HdataType,
        WeechatType::Info(_, _) => WeechatTypeEnum::InfoType,
        WeechatType::InfoList(_, _) => WeechatTypeEnum::InfoListType,
        WeechatType::Array(_) => WeechatTypeEnum::ArrayType,
    }
}

/// Get char value from a WeechatType::Char
/// @param weechat_type: WeechatType pointer
/// @return Char value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_char_get(weechat_type: *mut WeechatType) -> i8 {
    match *weechat_type {
        WeechatType::Char(c) => c,
        _ => 0
    }
}

/// Get int value from a WeechatType::Int
/// @param weechat_type: WeechatType pointer
/// @return Int value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_int_get(weechat_type: *mut WeechatType) -> i32 {
    match *weechat_type {
        WeechatType::Int(i) => i,
        _ => 0
    }
}

/// Get long value from a WeechatType::Long
/// @param weechat_type: WeechatType pointer
/// @return Long value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_long_get(weechat_type: *mut WeechatType) -> isize {
    match *weechat_type {
        WeechatType::Long(l) => l as isize,
        _ => 0
    }
}

/// Get string value from a WeechatType::String
/// @param weechat_type: WeechatType pointer
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_string_get(weechat_type: *mut WeechatType, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::String(WeechatString::Str(s)) => {
            *length = s.len();
            s.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get buffer value from a WeechatType::Buffer
/// @param weechat_type: WeechatType pointer
/// @param length: Pointer to length of buffer
/// @return Buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_buffer_get(weechat_type: *mut WeechatType, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::Buffer(Some(s)) => {
            *length = s.len();
            s.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get pointer value from a WeechatType::Pointer
/// @param weechat_type: WeechatType pointer
/// @return Pointer value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_pointer_get(weechat_type: *mut WeechatType) -> usize {
    match *weechat_type {
        WeechatType::Pointer(p) => p as usize,
        _ => 0
    }
}

/// Get time value from a WeechatType::Time
/// @param weechat_type: WeechatType pointer
/// @return Time value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_time_get(weechat_type: *mut WeechatType) -> usize {
    match *weechat_type {
        WeechatType::Time(t) => t as usize,
        _ => 0
    }
}

/// Get count of WeechatType::HashTable
/// @param weechat_type: WeechatType pointer
/// @return Number of entries in hash table
#[no_mangle]
pub unsafe extern "C" fn weechat_type_hash_table_count(weechat_type: *mut WeechatType) -> usize {
    match &*weechat_type {
        WeechatType::HashTable(htb) => {
            htb.len()
        },
        _ => 0
    }
}

/// Get entry key from a WeechatType::HashTable
/// @param weechat_type: WeechatType pointer
/// @param index: Index of entry
/// @return Entry key pointer
#[no_mangle]
pub unsafe extern "C" fn weechat_type_hash_table_get_key(weechat_type: *mut WeechatType, index: usize) -> *mut WeechatType {
    match &mut *weechat_type {
        WeechatType::HashTable(htb) => {
            &mut htb[index].0 as *mut WeechatType
        },
        _ => null_mut()
    }
}

/// Get entry value from a WeechatType::HashTable
/// @param weechat_type: WeechatType pointer
/// @param index: Index of entry
/// @return Entry value pointer
#[no_mangle]
pub unsafe extern "C" fn weechat_type_hash_table_get_value(weechat_type: *mut WeechatType, index: usize) -> *mut WeechatType {
    match &mut *weechat_type {
        WeechatType::HashTable(htb) => {
            &mut htb[index].1 as *mut WeechatType
        },
        _ => null_mut()
    }
}

/// Get pointer to an Hdata struct from a WeechatType::Hdata
/// @param weechat_type: WeechatType pointer
/// @return Pointer to Hdata struct
#[no_mangle]
pub unsafe extern "C" fn weechat_type_hdata_get(weechat_type: *mut WeechatType) -> *mut Hdata {
    match &mut *weechat_type {
        WeechatType::Hdata(hda) => {
            hda as *mut Hdata
        },
        _ => null_mut()
    }
}

/// Get info name from a WeechatType::Info
/// @param weechat_type: WeechatType pointer
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_get_name(weechat_type: *mut WeechatType, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::Info(WeechatString::Str(name), _) => {
            *length = name.len();
            name.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get info value from a WeechatType::Info
/// @param weechat_type: WeechatType pointer
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_get_value(weechat_type: *mut WeechatType, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::Info(_, WeechatString::Str(value)) => {
            *length = value.len();
            value.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get name of a WeechatType::InfoList
/// @param weechat_type: WeechatType pointer
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_list_get_name(weechat_type: *mut WeechatType, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::InfoList(WeechatString::Str(name), _) => {
            *length = name.len();
            name.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get number of items in a WeechatType::InfoList
/// @param weechat_type: WeechatType pointer
/// @return Number of infolist items
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_list_count(weechat_type: *mut WeechatType) -> usize {
    match &*weechat_type {
        WeechatType::InfoList(_, v) => v.len(),
        _ => 0
    }
}

/// Get number of entries in an item in a WeechatType::InfoList
/// @param weechat_type: WeechatType pointer
/// @param index: Index of item
/// @return Number of item entries
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_list_item_count(weechat_type: *mut WeechatType, index: usize) -> usize {
    match &*weechat_type {
        WeechatType::InfoList(_, v) => v[index].len(),
        _ => 0
    }
}

/// Get entry name in an item in a WeechatType::InfoList
/// @param weechat_type: WeechatType pointer
/// @param item_index: Index of item
/// @param entry_index: Index of entry
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_list_item_item_get_name(weechat_type: *mut WeechatType, item_index: usize, entry_index: usize, length: *mut usize) -> *const u8 {
    match &*weechat_type {
        WeechatType::InfoList(_, v) => {
            *length = v[item_index][entry_index].0.len();
            v[item_index][entry_index].0.as_ptr()
        },
        _ => {
            *length = 0;
            null()
        }
    }
}

/// Get entry value in an item in a WeechatType::InfoList
/// @param weechat_type: WeechatType pointer
/// @param item_index: Index of item
/// @param entry_index: Index of entry
/// @return Pointer to entry value
#[no_mangle]
pub unsafe extern "C" fn weechat_type_info_list_item_item_get_value(weechat_type: *mut WeechatType, item_index: usize, entry_index: usize) -> *mut WeechatType {
    match &mut *weechat_type {
        WeechatType::InfoList(_, v) => {
            &mut v[item_index][entry_index].1 as *mut WeechatType
        },
        _ => null_mut()
    }
}

/// Get number of items in a WeechatType::Array
/// @param weechat_type: WeechatType pointer
/// @return Number of array items
#[no_mangle]
pub unsafe extern "C" fn weechat_type_array_count(weechat_type: *mut WeechatType) -> usize {
    match &*weechat_type {
        WeechatType::Array(a) => a.len(),
        _ => 0
    }
}

/// Get item in a WeechatType::Array
/// @param weechat_type: WeechatType pointer
/// @paarm index: Index of item in array
/// @return Pointer to array item
#[no_mangle]
pub unsafe extern "C" fn weechat_type_array_item(weechat_type: *mut WeechatType, index: usize) -> *mut WeechatType {
    match &mut *weechat_type {
        WeechatType::Array(a) => {
            match a.get_mut(index) {
                Some(i) => {
                    i as *mut WeechatType
                }
                _ => null_mut()
            }
        },
        _ => null_mut()
    }
}

//-----------------------------------------------------------------------------

/// Get count of h_path in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @return Number of items in hdata's h_path
#[no_mangle]
pub unsafe extern "C" fn hdata_path_count(hdata: *mut Hdata) -> usize {
    (*hdata).h_path.len()
}

/// Get item of h_path in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @param index: Index of item in path
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn hdata_path_item(hdata: *mut Hdata, index: usize, length: *mut usize) -> *const u8 {
    *length = (*hdata).h_path[index].len();
    (*hdata).h_path[index].as_ptr()
}

/// Get count of keys in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @return Number of keys in hdata
#[no_mangle]
pub unsafe extern "C" fn hdata_keys_count(hdata: *mut Hdata) -> usize {
    (*hdata).keys.len()
}

/// Get name of key in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @param index: Index of key
/// @param length: Pointer to length of string
/// @return String buffer (not null-terminated)
#[no_mangle]
pub unsafe extern "C" fn hdata_keys_item(hdata: *mut Hdata, index: usize, length: *mut usize) -> *const u8 {
    *length = (*hdata).keys[index].0.len();
    (*hdata).keys[index].0.as_ptr()
}

/// Get count of buffers in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @return Number of buffers in hdata
#[no_mangle]
pub unsafe extern "C" fn hdata_buffer_count(hdata: *mut Hdata) -> usize {
    (*hdata).values.len()
}

/// Get buffer path item in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @param buffer_index: Index of buffer
/// @param path_index: Index of path item
/// @return Pointer to path item
#[no_mangle]
pub unsafe extern "C" fn hdata_buffer_path_item(hdata: *mut Hdata, buffer_index: usize, path_index: usize) -> *mut WeechatType {
    &mut (*hdata).values[buffer_index].0[path_index] as *mut WeechatType
}

/// Get buffer object in an Hdata
/// @param hdata: Pointer to Hdata struct
/// @param buffer_index: Index of buffer
/// @param key_index: Index of key
/// @return Pointer to object item
#[no_mangle]
pub unsafe extern "C" fn hdata_buffer_object_item(hdata: *mut Hdata, buffer_index: usize, key_index: usize) -> *mut WeechatType {
    &mut (*hdata).values[buffer_index].1[key_index] as *mut WeechatType
}

//-----------------------------------------------------------------------------

