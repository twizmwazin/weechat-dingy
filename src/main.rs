extern crate byteorder;

use std::io::*;
use std::net::TcpStream;

pub mod command;
pub mod message;

fn main() {
    let password = "something_long_and_hard_to_guess";
    let mabye_stream = TcpStream::connect("142.44.242.172:34513");

    if mabye_stream.is_err() {
        println!("Problems here");
        return;
    }

    let mut stream = mabye_stream.unwrap();

    let c1 = command::Command {
        id: None,
        command_type: command::CommandType::Init,
        args: vec!(String::from("password=something_long_and_hard_to_guess,compression=off")),
    };
    c1.encode(&mut stream).unwrap();
    let c2 = command::Command {
        id: None,
        command_type: command::CommandType::Info,
        args: vec!(String::from("version")),
    };
    c2.encode(&mut stream).unwrap();

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
