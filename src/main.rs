#[macro_use]
extern crate derive_more;
extern crate bytes;
extern crate futures;
extern crate rand;
extern crate tokio;

use crate::command::Command;
use crate::server::WeechatServer;
use std::env;
use std::net::ToSocketAddrs;
use std::thread;
use tokio::prelude::*;

mod codec;
pub mod command;
pub mod message;
pub mod server;
pub mod sync;

fn main() {
    let env_server_addr = env::var("server");
    let env_password = env::var("password");

    let server_addr = if let Some(server_addr) = env_server_addr
        .ok()
        .and_then(|addr| addr.to_socket_addrs().ok())
        .and_then(|mut addrs| addrs.next())
    {
        server_addr
    } else {
        println!("Need to define env server=<addr:port>");
        return;
    };
    let password = if let Ok(password) = env_password {
        password
    } else {
        println!("Need to define env password=relay_passwd");
        return;
    };

    println!("Addr: {:?}", server_addr);
    let server = WeechatServer::new(&server_addr);

    let init_command = command::InitCommand::new(
        Some("login".into()),
        Some(password.to_owned()),
        Some(command::CompressionType::None),
    );
    init_command.encode(&mut std::io::stdout()).unwrap();

    let sync = server.sync();

    let init_task = server
        .send(init_command)
        .and_then(|(tx, _)| {
            // Send stuff on separate thread.
            thread::spawn(move || {
                // TODO: Move test somewhere else
                let test_command = command::TestCommand::new(Some("aaa".into()));
                test_command.encode(&mut std::io::stdout()).unwrap();
                let commands_task = tx
                    .send(test_command)
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let ping_command = command::PingCommand::new(
                            None,
                            Some(vec!["abcdefg".into()]),
                        );
                        ping_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(ping_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let info_command =
                            command::InfoCommand::new(None, "version".to_owned());
                        info_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(info_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

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
                        tx.send(hdata_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let nick_command = command::NicklistCommand::new(
                            Some("nicks".to_owned()),
                            None,
                        );
                        nick_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(nick_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let sync_command = command::SyncCommand::new(None, vec![]);
                        sync_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(sync_command)
                    })
                    .then(|_| Ok(()));

                tokio::run(commands_task);
            });

            Ok(())
        })
        .map_err(|_| ())
        .join(
            sync.for_each(|msg| {
                let items = sync::SyncMessage::parse(&msg).unwrap();
                for vec in items {
                    for m in vec {
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

                Ok(())
            })
            .map_err(|_| ()),
        )
        .then(|_| Ok(()));

    tokio::run(init_task);
}
