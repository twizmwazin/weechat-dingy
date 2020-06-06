#[macro_use]
extern crate derive_more;
extern crate bytes;
extern crate futures;
extern crate rand;
extern crate tokio;
extern crate libdingy;

use libdingy::command::*;
use libdingy::sync::*;
use crate::server::CommandSender;
use crate::server::WeechatServer;
use futures::future::lazy;
use futures::sync::mpsc;
use std::env;
use std::io;
use std::net::ToSocketAddrs;
use std::thread;
use tokio::prelude::*;
use libdingy::command::CommandType::Infolist;

mod codec;
pub mod server;

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

    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));
    let stdin_rx = stdin_rx.map_err(|_| panic!("errors not possible on rx"));

    println!("Addr: {:?}", server_addr);
    let server = WeechatServer::new(&server_addr);

    let init_command = InitCommand::new(
        Some("login".into()),
        Some(password.to_owned()),
        Some(CompressionType::None),
    );
    init_command.encode(&mut std::io::stdout()).unwrap();

    let send_task = stdin_rx
        .fold(server.sender(), |tx, data| {
            let fut: Box<Future<Item = CommandSender, Error = ()> + Send> =
                if let Ok(s) = String::from_utf8(data) {
                    if let Some(spot) = s.find(" ") {
                        let (buffer, _) = s.split_at(spot);
                        let (_, message) = s.split_at(spot + 1);

                        let input_command = InputCommand::new(
                            None,
                            buffer.into(),
                            message.into(),
                        );
                        input_command.encode(&mut std::io::stdout()).unwrap();
                        Box::new(tx.send(input_command).map(|(tx, _)| tx))
                    } else {
                        Box::new(lazy(|| Ok(tx)))
                    }
                } else {
                    Box::new(lazy(|| Ok(tx)))
                };

            fut
        })
        .map_err(|_| ());

    let sync = server.sync();
    let init_task = server
        .send(init_command)
        .and_then(|(tx, _)| {
            // Send stuff on separate thread.
            thread::spawn(move || {
                // TODO: Move test somewhere else
                let test_command = TestCommand::new(Some("aaa".into()));
                test_command.encode(&mut std::io::stdout()).unwrap();
                let commands_task = tx
                    .send(test_command)
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let sync_command = SyncCommand::new(None, vec![]);
                        sync_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(sync_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let ping_command = PingCommand::new(
                            None,
                            Some(vec!["abcdefg".into()]),
                        );
                        ping_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(ping_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let inl_command = InfoListCommand::new(
                            None,
                            "buffer".into(),
                            None,
                            None
                        );
                        inl_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(inl_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let info_command =
                            InfoCommand::new(None, "version".to_owned());
                        info_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(info_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let hdata_command = HdataCommand::new(
                            Some("HDATA HERE".to_owned()),
                            "buffer".into(),
                            (
                                "gui_buffers".into(),
                                Some(HdataCommandLength::Infinite),
                            ),
                            vec![],
                            Some(vec!["number".into(), "name".into()]),
                        );
                        hdata_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(hdata_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        let nick_command = NicklistCommand::new(
                            Some("nicks".to_owned()),
                            None,
                        );
                        nick_command.encode(&mut std::io::stdout()).unwrap();
                        tx.send(nick_command)
                    })
                    .and_then(|(tx, msg)| {
                        println!("Got message: {:?}", msg);

                        Ok(())
                    })
                    .then(|_| Ok(()));

                tokio::run(commands_task);
            });

            Ok(())
        })
        .map_err(|_| ())
        .join(
            sync.for_each(|syncs| {
                println!("Sync message:");
                for m in &*syncs {
                    match m {
                        SyncMessage::BufferLineAdded(bla) => {
                            println!(
                                "<{}>: {}",
                                bla.prefix.to_str(),
                                bla.message.to_str()
                            );
                        }
                        SyncMessage::Nicklist(nl) => {
                            println!("{:?}", nl);
                        }
                        _ => {
                            println!("{:?}", m);
                        }
                    }
                }

                Ok(())
            })
            .map_err(|_| ()),
        )
        .join(send_task)
        .then(|_| Ok(()));

    tokio::run(init_task);
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
fn read_stdin(mut tx: mpsc::Sender<Vec<u8>>) {
    let mut stdin = io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf) {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx = match tx.send(buf).wait() {
            Ok(tx) => tx,
            Err(_) => break,
        };
    }
}
