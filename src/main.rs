#[macro_use]
extern crate derive_more;
extern crate tokio;

use std::env;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;
use crate::command::CommandString;

pub mod command;
pub mod message;
pub mod sync;

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

    println!("Addr: {:?}", env_server_addr.clone().unwrap());

    let server_addr = env_server_addr.expect("1").to_socket_addrs().expect("2").next().expect("3");
    let password = env_password.unwrap();

    let maybe_stream = TcpStream::connect(&server_addr).and_then(move |stream| {
        let init_command = command::InitCommand::new(
            None,
            Some(password.to_owned()),
            Some(command::CompressionType::None),
        );
        init_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, init_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        // TODO: Move test somewhere else
        let test_command = command::TestCommand::new(Some("aaa".into()));
        test_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, test_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        let ping_command = command::PingCommand::new(None, Some(vec!["abcdefg".into()]));
        ping_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, ping_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        let info_command = command::InfoCommand::new(None, "version".to_owned());
        info_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, info_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        let hdata_command = command::HdataCommand::new(
            Some("HDATA HERE".to_owned()),
            "buffer".into(),
            (
                "gui_buffers".into(),
                Some(command::HdataCommandLength::Infinite),
            ),
            vec![],
            Some(vec!["number".into(), "name".into()]),
        );
        hdata_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, hdata_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        let nick_command = command::NicklistCommand::new(Some("nicks".to_owned()), None);
        nick_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, nick_command.into_string().unwrap())
    }).and_then(|(stream, _)| {
        let sync_command = command::SyncCommand::new(None, vec![]);
        sync_command.encode(&mut std::io::stdout()).unwrap();
        io::write_all(stream, sync_command.into_string().unwrap())
    }).and_then(|(mut stream, _)| {
        loop {
            match message::Message::parse(&mut stream) {
                Ok(msg) => {
                    if !msg.id.is_empty() && &msg.id[0..1] == "_" {
                        let syncs = sync::SyncMessage::parse(&msg);

                        match syncs {
                            Ok(items) => {
                                for vec in items {
                                    for m in vec {
                                        println!("{:?}", m);
                                        match m {
                                            sync::SyncMessage::BufferLineAdded(bla) => {
                                                println!(
                                                    "<{}>: {}",
                                                    bla.prefix.to_str(),
                                                    bla.message.to_str()
                                                );
                                            }
                                            sync::SyncMessage::Nicklist(nl) => {
                                                println!("{:?}", nl);
                                            }
                                            _ => {
                                                println!("{:?}", m);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("{:?}", e);
                            }
                        }
                    } else {
                        println!("{:?}", msg);
                    }
                }
                Err(e) => {
                    println!("parse error");
                    println!("{:?}", e);
                }
            }
        }

        Ok(())
    }).map_err(|err| {
        println!("Connection error: {:?}", err);
    });

    tokio::run(maybe_stream);
}
