use std::io::{Error, Write};
use std::result::Result;
use std::string::String;
use std::vec::Vec;

pub enum CompressionType {
    Zlib,
    Off,
}

impl CompressionType {
    pub fn as_str(&self) -> &str {
        match self {
            &CompressionType::Zlib => "zlib",
            &CompressionType::Off => "off",
        }
    }
}

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
            &CommandType::Init => "init",
            &CommandType::Hdata => "hdata",
            &CommandType::Info => "info",
            &CommandType::Infolist => "infolist",
            &CommandType::Nicklist => "nicklist",
            &CommandType::Input => "input",
            &CommandType::Sync => "sync",
            &CommandType::Desync => "desync",
            &CommandType::Quit => "quit",
        }
    }

    pub fn from_string(s: &str) -> Result<CommandType, &'static str> {
        match s {
            "init" => Ok(CommandType::Init),
            "hdata" => Ok(CommandType::Hdata),
            "info" => Ok(CommandType::Info),
            "infolist" => Ok(CommandType::Infolist),
            "nicklist" => Ok(CommandType::Nicklist),
            "input" => Ok(CommandType::Input),
            "sync" => Ok(CommandType::Sync),
            "desync" => Ok(CommandType::Desync),
            "quit" => Ok(CommandType::Quit),
            _ => Err("No match"),
        }
    }
}

pub struct Command<'a> {
    pub id: Option<&'a str>,
    pub command_type: CommandType,
    pub args: Vec<String>,
}

impl<'a> Command<'a> {
    pub fn encode(&self, writer: &mut Write) -> Result<usize, Error> {
        let mut str_res = String::default();
        if self.id.is_some() {
            str_res.push_str(&format!("({}) ", self.id.unwrap()));
        }
        str_res.push_str(self.command_type.as_str());
        for arg in &self.args {
            str_res.push_str(" ");
            str_res.push_str(arg);
        }
        str_res.push('\n');
        writer.write(str_res.as_bytes())
    }

    pub fn new_init<'b>(
        id: Option<&'a str>,
        password: Option<&'b str>,
        compression: Option<CompressionType>,
    ) -> Command<'a> {
        let mut args = String::new();
        if password.is_some() {
            args.push_str(format!("password={}", password.unwrap()).as_str());
        }
        if compression.is_some() {
            args.push_str(format!("compression={}", compression.unwrap().as_str()).as_str());
        }
        Command {
            id: id,
            command_type: CommandType::Init,
            args: vec![args],
        }
    }
}
