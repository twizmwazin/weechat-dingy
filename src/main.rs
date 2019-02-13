extern crate backtrace;
extern crate byteorder;
#[macro_use]
extern crate derive_more;
extern crate libflate;

use std::env;
use std::net::TcpStream;

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

    let init_command = command::InitCommand::new(
        None,
        Some(password.to_owned()),
        Some(command::CompressionType::None),
    );
    init_command.encode(&mut std::io::stdout()).unwrap();
    init_command.encode(&mut stream).unwrap();

    let test_command = command::TestCommand::new(Some("aaa".into()));
    test_command.encode(&mut std::io::stdout()).unwrap();
    test_command.encode(&mut stream).unwrap();

    let ping_command = command::PingCommand::new(None, Some(vec!["abcdefg".into()]));
    ping_command.encode(&mut std::io::stdout()).unwrap();
    ping_command.encode(&mut stream).unwrap();

    let info_command = command::InfoCommand::new(None, "version".to_owned());
    info_command.encode(&mut std::io::stdout()).unwrap();
    info_command.encode(&mut stream).unwrap();

    let hdata_command = command::HdataCommand::new(
        Some("HDATA HERE".to_owned()),
        "buffer".into(),
        ("gui_buffers".into(), Some(command::HdataCommandLength::Infinite)),
        vec![],
        Some(vec!["number".into(), "name".into()]),
    );
    hdata_command.encode(&mut std::io::stdout()).unwrap();
    hdata_command.encode(&mut stream).unwrap();

    loop {
        match message::Message::parse(&mut stream) {
            Ok(msg) => {
                println!("{:?}", msg);
            }
            Err(e) => {
                println!("parse error");
                println!("{:?}", e);
            }
        }
    }
}
