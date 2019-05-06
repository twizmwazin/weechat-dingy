use crate::codec::WeechatCodec;
use crate::command::Command;
use crate::message::Message;
use crate::sync::SyncMessage;
use futures::future::*;
use futures::stream::iter_ok;
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

// Weechat server connection
pub struct WeechatServer {
    command_tx: Sender<BoxCommand>,
    pending: Arc<Mutex<PendingList>>,
}

// Future for sent commands, returned by .send()
// Future param is a tuple (tx: CommandSender, msg: Message)
pub struct SendCommand {
    id: String,
    tx: Sender<BoxCommand>,
    has_response: bool,
    pending: Arc<Mutex<PendingList>>,
}

// Helper class for sending commands in futures (for chaining)
#[derive(Clone)]
pub struct CommandSender {
    tx: Sender<BoxCommand>,
    pending: Arc<Mutex<PendingList>>,
}

// Private mutable state for pending data
struct PendingList {
    messages: HashMap<String, Message>,
    commands: HashSet<String>,
    tasks: Vec<Task>,
    receivers: Vec<Sender<Arc<Vec<SyncMessage>>>>,
}

impl PendingList {
    pub fn new() -> PendingList {
        PendingList {
            messages: HashMap::<String, Message>::new(),
            commands: HashSet::<String>::new(),
            tasks: Vec::<Task>::new(),
            receivers: Vec::<Sender<Arc<Vec<SyncMessage>>>>::new(),
        }
    }
}

impl CommandSender {
    pub fn send<C: Command + Send + 'static>(
        self,
        mut command: C,
    ) -> impl Future<Item = (CommandSender, Option<Message>), Error = ()> {
        let id = if let Some(id) = command.get_id() {
            id
        } else {
            command.set_id(Some(self.generate_id()));
            command.get_id().unwrap()
        };
        let pending = self.pending.clone();
        let has_response = command.has_response();

        if has_response {
            let mut mpending = self.pending.lock().unwrap();
            assert_eq!(mpending.commands.insert(id.clone()), true);
        }

        self.tx.send(Box::new(command)).map_err(|_| ()).and_then(move |tx| {
            SendCommand { id, tx, has_response, pending }.map_err(|_| ())
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
    type Item = (CommandSender, Option<Message>);
    type Error = std::io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.has_response {
            return Ok(Async::Ready((
                CommandSender { tx: self.tx.clone(), pending: self.pending.clone() },
                None,
            )));
        }

        let mut mpending = self.pending.lock().unwrap();
        mpending.tasks.push(task::current());

        if let Some(msg) = mpending.messages.remove(&self.id) {
            Ok(Async::Ready((
                CommandSender { tx: self.tx.clone(), pending: self.pending.clone() },
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

        let pending = Arc::new(Mutex::new(PendingList::new()));

        let future = WeechatServer::start(
            addr,
            command_rx,
            message_rx,
            message_tx,
            pending.clone(),
        );

        thread::spawn(move || {
            tokio::run(future);
        });

        WeechatServer { command_tx, pending }
    }

    pub fn send<C: Command + Send + 'static>(
        &self,
        command: C,
    ) -> impl Future<Item = (CommandSender, Option<Message>), Error = ()> {
        CommandSender { tx: self.command_tx.clone(), pending: self.pending.clone() }
            .send(command)
    }

    pub fn sender(&self) -> CommandSender {
        CommandSender { tx: self.command_tx.clone(), pending: self.pending.clone() }
    }

    pub fn sync(&self) -> Receiver<Arc<Vec<SyncMessage>>> {
        let (tx, rx) = mpsc::channel::<Arc<Vec<SyncMessage>>>(0);

        let mut mpending = self.pending.lock().unwrap();
        mpending.receivers.push(tx);

        rx
    }

    fn start(
        addr: &SocketAddr,
        command_rx: Receiver<BoxCommand>,
        message_rx: Receiver<Message>,
        message_tx: Sender<Message>,
        pending: Arc<Mutex<PendingList>>,
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
                            .fold(pending, |pending, msg| {
                                WeechatServer::handle_message(msg, pending.clone())
                                    .map(|_| pending)
                            })
                            .map_err(|e| println!("Receive error: {:?}", e)),
                    )
                    .map_err(|e| println!("Join error: {:?}", e))
            })
            .map_err(|e| println!("Tcp error: {:?}", e));

        Box::new(tcp.map(|_| ()))
    }

    fn handle_message(
        msg: Message,
        pending: Arc<Mutex<PendingList>>,
    ) -> impl Future<Item = (), Error = ()> {
        // Sync messages start with an _ (except pongs are wild)
        let is_sync =
            !msg.id.is_empty() && &msg.id[0..1] == "_" && msg.id != "_pong";

        let mut mpending = pending.lock().unwrap();
        let mut futs = Vec::<Box<Future<Item = (), Error = ()> + Send>>::new();

        if is_sync {
            let items = SyncMessage::parse(&msg).unwrap();
            for vec in items {
                let mut inner_futs =
                    Vec::<Box<Future<Item = (), Error = ()> + Send>>::new();
                let arc = Arc::<Vec<SyncMessage>>::new(vec);

                for receiver in &mpending.receivers {
                    inner_futs.push(Box::new(
                        receiver.clone().send(arc.clone()).then(|_| Ok(())),
                    ));
                }

                futs.push(Box::new(join_all(inner_futs).then(|_| Ok(()))));
            }
        } else {
            if !mpending.commands.remove(&msg.id) {
                println!("Unexpected command response: {:?}", msg);
            }
            mpending.messages.insert(msg.id.clone(), msg);
            for task in mpending.tasks.iter() {
                task.notify();
            }
            mpending.tasks.clear();
        }

        iter_ok(futs).for_each(|fut| fut)
    }
}
