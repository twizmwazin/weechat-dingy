extern crate byteorder;
#[macro_use]
extern crate derive_more;
extern crate libflate;

use std::io::*;
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

    let mut buf: [u8; 256] = [0; 256];
    let mut rcv_cnt = 0;

    let readed = stream.read(&mut buf);

    if readed.is_err() {
        println!("Read error lmao");
        return;
    }

    rcv_cnt = readed.unwrap_or(0);
    let buf_slice = &buf[0..rcv_cnt];
    let msg = message::RelayMessage::from_bytes(buf_slice);
}
