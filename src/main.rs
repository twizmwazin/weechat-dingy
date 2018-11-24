extern crate byteorder;
#[macro_use]
extern crate derive_more;
extern crate libflate;

use std::net::TcpStream;
use std::env;

pub mod command;

pub mod message;

fn main() {
    let env_server_addr = env::var("server");
    let env_password = env::var("password");

    if env_server_addr.is_err() {
        println!("Need to define env server=<addr:port>");
        return;
    }
    if env_password.is_err() {
        println!("Need to define env password=relay_passwd");
        return;
    }

    let server_addr = env_server_addr.unwrap();
    let password = env_password.unwrap();

    let maybe_stream = TcpStream::connect(server_addr);

    if maybe_stream.is_err() {
        println!("Problems here");
        return;
    }

    let mut stream = maybe_stream.unwrap();

    command::InitCommand::new(
        None,
        Some(password.to_owned()),
        Some(command::CompressionType::None),
    ).encode(&mut stream)
    .unwrap();
    command::InfoCommand::new(None, "version".to_owned())
        .encode(&mut stream)
        .unwrap();

    let msg_res = message::Message::parse(&mut stream);
    if msg_res.is_err() {
        println!("Parse error");
        return;
    }

    let messages = msg_res.ok().unwrap().data;
    for msg in messages {
        match msg {
            message::WeechatType::Char(_) => {},
            message::WeechatType::Int(_) => {},
            message::WeechatType::Long(_) => {},
            message::WeechatType::String(_) => {},
            message::WeechatType::Buffer(_) => {},
            message::WeechatType::Pointer(_) => {},
            message::WeechatType::Time(_) => {},
            message::WeechatType::HashTable(_) => {},
            message::WeechatType::Hdata(_) => {},
            message::WeechatType::Info(name, value) => {
                println!("Got info: {} = {}", name, value);
            },
            message::WeechatType::InfoList(_, _) => {},
            message::WeechatType::Array(_) => {},
        }
    }
}
