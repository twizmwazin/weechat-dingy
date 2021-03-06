use std::io::Error;
use std::io::Write;
use std::option::Option;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

#[derive(Copy)]
pub enum CommandType {
    Init,
    Hdata,
    Info,
    Infolist,
    Nicklist,
    Input,
    Sync,
    Desync,
    Quit,
}

impl CommandType {
    pub fn as_str(&self) -> &str {
        match self {
            CommandType::Init => "init",
            CommandType::Hdata => "hdata",
            CommandType::Info => "info",
            CommandType::Infolist => "infolist",
            CommandType::Nicklist => "nicklist",
            CommandType::Input => "input",
            CommandType::Sync => "sync",
            CommandType::Desync => "desync",
            CommandType::Quit => "quit",
        }
    }
}

impl Clone for CommandType {
    fn clone(&self) -> CommandType {
        *self
    }
}

pub trait Command {
    fn get_id(&self) -> Option<String>;
    fn set_id(&mut self, id: Option<String>);
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error>;
    fn has_response(&self) -> bool;
}

pub trait CommandString {
    fn into_string(self) -> Option<String>;
}

impl<T> CommandString for T
where
    T: Command,
{
    fn into_string(self) -> Option<String> {
        let mut value: Vec<u8> = vec![];
        self.encode(&mut value).ok().and_then(|_| String::from_utf8(value).ok())
    }
}

#[derive(Copy)]
#[repr(C)]
pub enum CompressionType {
    None,
    Zlib,
}

impl Clone for CompressionType {
    fn clone(&self) -> CompressionType {
        *self
    }
}

impl CompressionType {
    fn as_str(self) -> &'static str {
        match self {
            CompressionType::None => "none",
            CompressionType::Zlib => "zlib",
        }
    }
}

#[derive(Constructor)]
pub struct InitCommand {
    id: Option<String>,
    password: Option<String>,
    compression: Option<CompressionType>,
}

impl InitCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res = handle_id(&self.id);
        res.push_str("init");
        if self.password.is_some() || self.compression.is_some() {
            res.push_str(" ");
            if self.password.is_some() {
                res = format!(
                    "{}password={}",
                    res,
                    escape_password(&self.password.clone().unwrap())
                );
            }
            if self.compression.is_some() {
                if self.password.is_some() {
                    res.push_str(",")
                }
                res = format!(
                    "{}compression={}",
                    res,
                    self.compression.unwrap().as_str()
                );
            }
        }
        res.push_str("\n");
        out.write(res.as_bytes())
    }
}

impl Command for InitCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        false
    }
}

pub enum HdataCommandLength {
    Infinite,
    Finite(i32),
}

#[derive(Constructor)]
pub struct HdataCommand {
    id: Option<String>,
    hdata: String,
    pointer: (String, Option<HdataCommandLength>),
    var: Vec<(String, Option<HdataCommandLength>)>,
    keys: Option<Vec<String>>,
}

impl HdataCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res = handle_id(&self.id);
        res = format!("{}hdata {}:{}", res, self.hdata, self.pointer.0);

        if let Some(e) = &self.pointer.1 {
            res = match e {
                HdataCommandLength::Infinite => format!("{}(*)", res),
                HdataCommandLength::Finite(count) => format!("{}({})", res, count),
            }
        }
        //hdata buffer:gui_buffers(*) number,name

        for v in self.var.iter() {
            res = format!("{}/{}", res, v.0);
            if let Some(e) = &v.1 {
                res = match e {
                    HdataCommandLength::Infinite => format!("{}(*)", res),
                    HdataCommandLength::Finite(count) => {
                        format!("{}({})", res, count)
                    }
                }
            }
        }
        if self.keys.is_some() {
            res = format!("{} {}", res, self.keys.clone().unwrap().join(","));
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for HdataCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct InfoCommand {
    id: Option<String>,
    name: String,
}

impl InfoCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let res = format!("{}info {}\n", handle_id(&self.id), self.name);
        out.write(res.as_bytes())
    }
}

impl Command for InfoCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct InfoListCommand {
    id: Option<String>,
    name: String,
    pointer: Option<String>,
    arguments: Option<Vec<String>>,
}

impl InfoListCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res = handle_id(&self.id);
        res = format!("{}infolist {}", res, &self.name);
        if self.pointer.is_some() {
            res = format!("{} {}", res, &self.pointer.clone().unwrap());
            if self.arguments.is_some() {
                for arg in self.arguments.clone().unwrap() {
                    res = format!("{} {}", res, arg);
                }
            }
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for InfoListCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct NicklistCommand {
    id: Option<String>,
    buffer: Option<String>,
}

impl NicklistCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res = handle_id(&self.id);
        res.push_str("nicklist");
        if self.buffer.is_some() {
            res = format!("{} {}", res, self.buffer.clone().unwrap());
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for NicklistCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct InputCommand {
    id: Option<String>,
    buffer: String,
    data: String,
}

impl InputCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let res =
            format!("{}input {} {}\n", handle_id(&self.id), self.buffer, self.data);
        out.write(res.as_bytes())
    }
}

impl Command for InputCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        false
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum SyncOption {
    Buffers,
    Upgrade,
    Buffer,
    Nicklist,
}

impl SyncOption {
    fn as_str(&self) -> &'static str {
        match self {
            SyncOption::Buffers => "buffers",
            SyncOption::Upgrade => "upgrade",
            SyncOption::Buffer => "buffer",
            SyncOption::Nicklist => "nicklist",
        }
    }
}

#[derive(Constructor)]
pub struct SyncCommand {
    id: Option<String>,
    args: Vec<(String, SyncOption)>,
}

impl SyncCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res: String = handle_id(&self.id);
        res = format!("{}sync", res);
        if !self.args.is_empty() {
            res.push(' ');
            for (i, arg) in self.args.iter().enumerate() {
                if i != 0 {
                    res.push(',');
                }
                res.push_str(arg.0.as_str());
            }
            res.push(' ');
            for (i, arg) in self.args.iter().enumerate() {
                if i != 0 {
                    res.push(',');
                }
                res.push_str(arg.1.as_str());
            }
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for SyncCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        false
    }
}

#[derive(Constructor)]
pub struct DesyncCommand {
    id: Option<String>,
    args: Vec<(String, SyncOption)>,
}

impl DesyncCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res: String = handle_id(&self.id);
        res = format!("{}desync", res);
        if !self.args.is_empty() {
            res.push(' ');
            for (i, arg) in self.args.iter().enumerate() {
                if i != 0 {
                    res.push(',');
                }
                res.push_str(arg.0.as_str());
            }
            res.push(' ');
            for (i, arg) in self.args.iter().enumerate() {
                if i != 0 {
                    res.push(',');
                }
                res.push_str(arg.1.as_str());
            }
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for DesyncCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        false
    }
}

#[derive(Constructor)]
pub struct TestCommand {
    id: Option<String>,
}

impl TestCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let res = format!("{}test\n", handle_id(&self.id));
        out.write(res.as_bytes())
    }
}

impl Command for TestCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct PingCommand {
    id: Option<String>,
    arguments: Option<Vec<String>>,
}

impl PingCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let mut res = handle_id(&self.id);
        res.push_str("ping");
        if self.arguments.is_some() {
            for arg in self.arguments.clone().unwrap() {
                res.push_str(format!(" {}", arg).as_str());
            }
        }
        res.push('\n');
        out.write(res.as_bytes())
    }
}

impl Command for PingCommand {
    fn get_id(&self) -> Option<String> {
        Some("_pong".into())
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        true
    }
}

#[derive(Constructor)]
pub struct QuitCommand {
    id: Option<String>,
}

impl QuitCommand {
    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        let res = format!("{}quit\n", handle_id(&self.id));
        out.write(res.as_bytes())
    }
}

impl Command for QuitCommand {
    fn get_id(&self) -> Option<String> {
        self.id.clone()
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn encode(&self, out: &mut dyn Write) -> Result<usize, Error> {
        self.encode(out)
    }

    fn has_response(&self) -> bool {
        false
    }
}

// Helper functions

fn handle_id(id: &Option<String>) -> String {
    if id.is_some() {
        format!("({}) ", id.clone().unwrap())
    } else {
        String::new()
    }
}

fn escape_password(input: &str) -> String {
    // This implementation can probably be optimized
    let mut res = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            ',' => res.push_str("\\,"),
            _ => res.push(c),
        }
    }
    res
}
