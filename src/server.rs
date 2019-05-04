use crate::codec::WeechatCodec;
use crate::command::Command;
use crate::message::Message;
use futures::sync::mpsc;
use futures::sync::mpsc::*;
use futures::task::Task;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::vec::Vec;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;

type BoxCommand = Box<Command + Send>;
type PendingList = Arc<Mutex<HashMap<String, Message>>>;
type PendingCommandList = Arc<Mutex<HashSet<String>>>;
type PendingTaskList = Arc<Mutex<Vec<Task>>>;

pub struct WeechatServer {
    command_tx: Sender<BoxCommand>,
    pending: PendingList,
    pending_commands: PendingCommandList,
    pending_tasks: PendingTaskList,
}

pub struct SendCommand {
    id: String,
    tx: Sender<BoxCommand>,
    has_response: bool,
    pending: PendingList,
    pending_commands: PendingCommandList,
    pending_tasks: PendingTaskList,
}

pub struct ServerSender {
    tx: Sender<BoxCommand>,
    pending: PendingList,
    pending_commands: PendingCommandList,
    pending_tasks: PendingTaskList,
}

impl ServerSender {
    pub fn send<C: Command + Send + 'static>(
        self,
        mut command: C,
    ) -> impl Future<Item = (ServerSender, Option<Message>), Error = ()> {
        let id = if let Some(id) = command.get_id() {
            id
        } else {
            command.set_id(Some(self.generate_id()));
            command.get_id().unwrap()
        };
        let pending = self.pending.clone();
        let pending_tasks = self.pending_tasks.clone();
        let pending_commands = self.pending_commands.clone();
        let has_response = command.has_response();

        if has_response {
            assert_eq!(
                self.pending_commands.lock().unwrap().insert(id.clone()),
                true
            );
        }

        self.tx.send(Box::new(command)).map_err(|_| ()).and_then(move |tx| {
            SendCommand {
                id,
                tx,
                has_response,
                pending,
                pending_commands,
                pending_tasks,
            }
            .map_err(|_| ())
        })
    }

    fn generate_id(&self) -> String {
        let rand_id: String =
            thread_rng().sample_iter(&Alphanumeric).take(10).collect();

        return rand_id;
    }
}

impl Hash for SendCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Future for SendCommand {
    type Item = (ServerSender, Option<Message>);
    type Error = std::io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.has_response {
            return Ok(Async::Ready((
                ServerSender {
                    tx: self.tx.clone(),
                    pending: self.pending.clone(),
                    pending_commands: self.pending_commands.clone(),
                    pending_tasks: self.pending_tasks.clone(),
                },
                None,
            )));
        }

        let mut pending_tasks = self.pending_tasks.lock().unwrap();
        pending_tasks.push(task::current());

        let mut pending = self.pending.lock().unwrap();
        if let Some(msg) = pending.remove(&self.id) {
            Ok(Async::Ready((
                ServerSender {
                    tx: self.tx.clone(),
                    pending: self.pending.clone(),
                    pending_commands: self.pending_commands.clone(),
                    pending_tasks: self.pending_tasks.clone(),
                },
                Some(msg),
            )))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl WeechatServer {
    pub fn new(addr: &SocketAddr) -> WeechatServer {
        let (command_tx, command_rx) = mpsc::channel::<BoxCommand>(0);
        let (message_tx, message_rx) = mpsc::channel::<Message>(0);

        let pending = Arc::new(Mutex::new(HashMap::<String, Message>::new()));
        let pending_commands = Arc::new(Mutex::new(HashSet::<String>::new()));
        let pending_tasks = Arc::new(Mutex::new(Vec::<Task>::new()));

        let future = WeechatServer::start(
            addr,
            command_rx,
            message_rx,
            message_tx,
            pending.clone(),
            pending_commands.clone(),
            pending_tasks.clone(),
        );

        thread::spawn(move || {
            tokio::run(future);
        });

        WeechatServer { command_tx, pending, pending_commands, pending_tasks }
    }

    pub fn send<C: Command + Send + 'static>(
        &self,
        command: C,
    ) -> impl Future<Item = (ServerSender, Option<Message>), Error = ()> {
        ServerSender {
            tx: self.command_tx.clone(),
            pending: self.pending.clone(),
            pending_commands: self.pending_commands.clone(),
            pending_tasks: self.pending_tasks.clone(),
        }
        .send(command)
    }

    fn start(
        addr: &SocketAddr,
        command_rx: Receiver<BoxCommand>,
        message_rx: Receiver<Message>,
        message_tx: Sender<Message>,
        pending: PendingList,
        pending_commands: PendingCommandList,
        pending_tasks: PendingTaskList,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        let tcp = TcpStream::connect(addr)
            .map_err(|e| println!("Connect failed: {:?}", e))
            .and_then(move |stream| {
                let (sink, stream) =
                    Framed::new(stream, WeechatCodec::new()).split();

                command_rx
                    .fold(sink, move |sink, command: Box<Command + Send>| {
                        sink.send(command).map_err(|err| {
                            println!("Send error: {:?}", err);
                        })
                    })
                    .join(
                        stream
                            .map_err(|e| println!("Stream read error: {:?}", e))
                            .fold(message_tx, |tx, msg| {
                                tx.send(msg).map_err(|e| {
                                    println!("Message tx error: {:?}", e)
                                })
                            })
                            .map_err(|e| println!("Fold error: {:?}", e)),
                    )
                    .join(
                        message_rx
                            .fold(
                                (pending, pending_commands, pending_tasks),
                                |(pending, pending_commands, pending_tasks), msg| {
                                    WeechatServer::handle_message(
                                        msg,
                                        pending.clone(),
                                        pending_commands.clone(),
                                        pending_tasks.clone(),
                                    )
                                    .map(|_| {
                                        (pending, pending_commands, pending_tasks)
                                    })
                                },
                            )
                            .map_err(|e| println!("Receive error: {:?}", e)),
                    )
                    .map_err(|e| println!("Join error: {:?}", e))
            })
            .map_err(|e| println!("Tcp error: {:?}", e));

        Box::new(tcp.map(|_| ()))
    }

    fn handle_message(
        msg: Message,
        pending: PendingList,
        pending_commands: PendingCommandList,
        pending_tasks: PendingTaskList,
    ) -> Result<(), ()> {
        let mut mpending_commands = pending_commands.lock().unwrap();
        mpending_commands.remove(&msg.id);

        let mut mpending = pending.lock().unwrap();
        mpending.insert(msg.id.clone(), msg);

        let mut mpending_tasks = pending_tasks.lock().unwrap();
        for task in mpending_tasks.iter() {
            task.notify();
        }
        mpending_tasks.clear();

        Ok(())
    }
}
