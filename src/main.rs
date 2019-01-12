extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::prelude::*;

type Message = String;
type ActorId = u64;

struct ActorAddress {
    id: ActorId,
    sender: Sender<Message>,
}
impl Hash for ActorAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ActorAddress {
    fn eq(&self, other: &ActorAddress) -> bool {
        self.id == other.id
    }
}
impl Eq for ActorAddress {}
impl ActorAddress {
    fn send(&self, message: Message) {
        println!("sending message");
        self.sender
            .clone()
            .send_all(stream::once(Ok(message)))
            .wait()
            .ok();
    }
}

trait Actor {}

struct Router {
    actors: HashMap<ActorId, Arc<Mutex<RootActor>>>,
    actor_id_counter: ActorId,
}
impl Router {
    fn new() -> Router {
        Router {
            actors: HashMap::new(),
            actor_id_counter: 0,
        }
    }

    fn register_actor(
        &mut self,
        actor: Arc<Mutex<RootActor>>,
    ) -> (ActorAddress, Receiver<Message>) {
        self.actor_id_counter += 1;
        self.actors.insert(self.actor_id_counter.clone(), actor);

        let (sender, receiver) = channel::<Message>(16);

        let address = ActorAddress {
            id: self.actor_id_counter.clone(),
            sender: sender,
        };
        (address, receiver)
    }
}

struct SystemMessage {
    message_type: SystemMessageType,
    payload: Option<String>,
}
enum SystemMessageType {
    STOP,
}

#[derive(Clone)]
struct Context {
    sender_register_actor: Sender<SystemMessage>,
    system: Arc<Mutex<ActorSystem>>,
}
impl Context {
    fn send_system_message(&self, system_message: SystemMessage) {
        self.sender_register_actor
            .clone()
            .send_all(stream::once(Ok(system_message)))
            .wait()
            .ok();
    }

    fn register_actor(&self, actor: Arc<Mutex<RootActor>>) -> ActorAddress {
        self.system.lock().unwrap().register_actor(actor)
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn run_setup<F>(f: F, context: Context)
    where
        F: FnOnce(Context) + 'static + Send,
    {
        tokio::run(future::ok(()).map(move |_| {
            println!("setting up system");
            f(context);
        }));
    }

    fn register_mailbox(&self, receiver: Receiver<Message>, actor: Arc<Mutex<RootActor>>) {
        tokio::spawn(
            receiver
                .map(move |message: Message| {
                    actor.lock().unwrap().handle(message);
                })
                .collect()
                .then(|_| Ok(())),
        );
    }
}

struct ActorSystem {
    dispatcher: Dispatcher,
    router: Router,
}
impl ActorSystem {
    fn create() -> ActorSystem {
        println!("creating ActorSystem");
        ActorSystem {
            dispatcher: Dispatcher {},
            router: Router::new(),
        }
    }

    fn start<F>(f: F)
    where
        F: FnOnce(Context) + 'static + Send,
    {
        // establish system messae channel
        let (system_sender, system_receiver) = channel::<SystemMessage>(64);

        // init actor system
        let actor_system = Arc::new(Mutex::new(ActorSystem::create()));
        let actor_system_clone = actor_system.clone();

        let context = Context {
            sender_register_actor: system_sender.clone(),
            system: actor_system.clone(),
        };

        Dispatcher::run_setup(f, context.clone());

        // spawn system message handler task
        tokio::run(
            system_receiver
                .map(move |message: SystemMessage| {
                    println!("handling system message");
                    actor_system_clone.lock().unwrap().handle(message);
                })
                .collect()
                .then(|_| Ok(())),
        );
    }
    fn register_actor(&mut self, actor: Arc<Mutex<RootActor>>) -> ActorAddress {
        println!("registering new actor: {}", actor.lock().unwrap().name);
        let (address, mailbox_receiver) = self.router.register_actor(actor.clone());
        self.dispatcher
            .register_mailbox(mailbox_receiver, actor.clone());
        address
    }
}
impl Handler<SystemMessage> for ActorSystem {
    fn handle(&mut self, message: SystemMessage) {
        match message.message_type {
            SystemMessageType::STOP => {
                println!("!!! received STOP system message");
                // TODO: somehow stop system loop
                println!("doesn't work right now though");
            }
            _ => println!("!!! received unknown system message"),
        }
    }
}

trait Handler<T> {
    fn handle(&mut self, message: T);
}

struct RootActor {
    name: String,
}
impl Actor for RootActor {}
impl Handler<Message> for RootActor {
    fn handle(&mut self, message: Message) {
        println!("root actor received message: {}", message);
    }
}

fn main() {
    println!("init");
    ActorSystem::start(|context: Context| {
        println!("setup closure");
        let root_actor = Arc::new(Mutex::new(RootActor {
            name: "root actor".to_owned(),
        }));
        let addr = context.register_actor(root_actor);

        tokio::spawn(future::ok(()).map(move |_| {
            thread::sleep(Duration::from_millis(1000));
            addr.send("message 1".to_owned());
            thread::sleep(Duration::from_millis(2000));
            addr.send("message 2".to_owned());
            thread::sleep(Duration::from_millis(2000));
            context.send_system_message(SystemMessage {
                message_type: SystemMessageType::STOP,
                payload: None,
            });
        }));
    });
    println!("done");
}
