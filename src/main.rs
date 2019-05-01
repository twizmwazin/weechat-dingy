#[macro_use]
extern crate derive_more;
extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::codec::WeechatCodec;
use crate::command::Command;
use futures::sync::mpsc;
use std::env;
use std::net::ToSocketAddrs;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub mod codec;
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

    let server_addr = env_server_addr
        .expect("1")
        .to_socket_addrs()
        .expect("2")
        .next()
        .expect("3");
    let password = env_password.unwrap();

    //Goal API:
    // init_command.write(stream).and_then(|(stream, message)| {
    //     //Deal with message here
    //     Ok()
    // })

    let (tx, rx) = mpsc::channel::<Box<Command + Send>>(0);

    let rx = rx.map_err(|_| panic!(""));

    let tcp = TcpStream::connect(&server_addr)
        .and_then(move |stream| {
            println!("Ah");
            let (sink, stream) = Framed::new(stream, WeechatCodec::new()).split();

            let init_command = command::InitCommand::new(
                None,
                Some(password.to_owned()),
                Some(command::CompressionType::None),
            );
            init_command.encode(&mut std::io::stdout()).unwrap();
            let task = tx
                .clone()
                .send(Box::new(init_command))
                .and_then(|tx| {
                    // TODO: Move test somewhere else
                    let test_command = command::TestCommand::new(Some("aaa".into()));
                    test_command.encode(&mut std::io::stdout()).unwrap();
                    tx.send(Box::new(test_command))
                })
                .and_then(|tx| {
                    let ping_command =
                        command::PingCommand::new(None, Some(vec!["abcdefg".into()]));
                    ping_command.encode(&mut std::io::stdout()).unwrap();
                    tx.send(Box::new(ping_command))
                })
                .and_then(|tx| {
                    let info_command = command::InfoCommand::new(None, "version".to_owned());
                    info_command.encode(&mut std::io::stdout()).unwrap();
                    tx.send(Box::new(info_command))
                })
                .and_then(|tx| {
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
                    tx.send(Box::new(hdata_command))
                })
                .and_then(|tx| {
                    let nick_command =
                        command::NicklistCommand::new(Some("nicks".to_owned()), None);
                    nick_command.encode(&mut std::io::stdout()).unwrap();
                    tx.send(Box::new(nick_command))
                })
                .and_then(|tx| {
                    let sync_command = command::SyncCommand::new(None, vec![]);
                    sync_command.encode(&mut std::io::stdout()).unwrap();
                    tx.send(Box::new(sync_command))
                })
                .then(|_| Ok(()));

            let task2 = rx
                .fold(sink, move |sink, command: Box<Command + Send>| {
                    sink.send(command).map_err(|err| {
                        println!("Send error: {:?}", err);
                    })
                })
                .then(|_| Ok(()));

            tokio::spawn(task.join(task2).map(|_| ()));

            stream.for_each(|msg| {
                if !msg.id.is_empty() && &msg.id[0..1] == "_" {
                    let items = sync::SyncMessage::parse(&msg)?;

                    for vec in items {
                        for m in vec {
                            println!("{:?}", m);
                            match m {
                                sync::SyncMessage::BufferLineAdded(bla) => {
                                    println!("<{}>: {}", bla.prefix.to_str(), bla.message.to_str());
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
                } else {
                    println!("{:?}", msg);
                }

                Ok(())
            })
        })
        .map_err(|e| println!("{:?}", e));

    tokio::run(tcp);
}
