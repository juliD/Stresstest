extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::prelude::*;

trait Message {}
type ActorId = u64;

#[derive(Clone)]
struct ActorAddress {
    id: ActorId,
    sender: Sender<Box<Message + Send>>,
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
    fn send(&self, message: Box<Message + Send>) {
        self.sender
            .clone()
            .send_all(stream::once(Ok(message)))
            .wait()
            .ok();
    }
}

trait Actor {
    fn handle(&mut self, message: Box<Message + Send>, context: Context);
}

struct Router {
    actors: HashMap<ActorId, Arc<Mutex<Actor + Send>>>,
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
        actor: Arc<Mutex<Actor + Send>>,
    ) -> (ActorAddress, Receiver<Box<Message + Send>>) {
        self.actor_id_counter += 1;
        self.actors.insert(self.actor_id_counter.clone(), actor);

        let (sender, receiver) = channel::<Box<Message + Send>>(16);

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

    fn register_actor(&self, actor: Arc<Mutex<Actor + Send>>) -> ActorAddress {
        self.system
            .lock()
            .unwrap()
            .register_actor(actor, self.clone())
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

    fn register_mailbox(
        &self,
        receiver: Receiver<Box<Message + Send>>,
        actor: Arc<Mutex<Actor + Send>>,
        context: Context,
    ) {
        tokio::spawn(
            receiver
                .map(move |message: Box<Message + Send>| {
                    actor.lock().unwrap().handle(message, context.clone());
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
        // establish system message channel
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
                    actor_system_clone
                        .lock()
                        .unwrap()
                        .handle_system_message(message);
                })
                .collect()
                .then(|_| Ok(())),
        );
    }
    fn register_actor(
        &mut self,
        actor: Arc<Mutex<Actor + Send>>,
        context: Context,
    ) -> ActorAddress {
        // println!("registering new actor: {}", actor.lock().unwrap().name);
        let (address, mailbox_receiver) = self.router.register_actor(actor.clone());
        self.dispatcher
            .register_mailbox(mailbox_receiver, actor.clone(), context.clone());
        address
    }

    fn handle_system_message(&mut self, message: SystemMessage) {
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

struct SomeActor {}
impl Actor for SomeActor {
    fn handle(&mut self, message: Box<Message + Send>, context: Context) {
        println!("some actor received a message");
    }
}

struct OtherActor {
    target: ActorAddress,
}
impl Actor for OtherActor {
    fn handle(&mut self, message: Box<Message + Send>, context: Context) {
        println!("other actor received a message");
        match message {
            HelloMessage => {
                println!("  It's a HelloMessage");
                self.target.send(Box::new(HelloMessage {
                    greeting: "Hi".to_owned(),
                }));
            },
        }
    }
}

struct HelloMessage {
    greeting: String,
}
impl Message for HelloMessage {}

fn main() {
    println!("init");
    ActorSystem::start(|context: Context| {
        let some_actor = Arc::new(Mutex::new(SomeActor {}));
        let some_addr = context.register_actor(some_actor);

        let other_actor = Arc::new(Mutex::new(OtherActor {
            target: some_addr.clone(),
        }));
        let other_addr = context.register_actor(other_actor);

        tokio::spawn(future::ok(()).map(move |_| {
            thread::sleep(Duration::from_millis(1000));
            other_addr.send(Box::new(HelloMessage {
                greeting: "Hi".to_owned(),
            }));
            thread::sleep(Duration::from_millis(2000));
            other_addr.send(Box::new(HelloMessage {
                greeting: "Hello".to_owned(),
            }));
            // thread::sleep(Duration::from_millis(2000));
            // context.send_system_message(SystemMessage {
            //     message_type: SystemMessageType::STOP,
            //     payload: None,
            // });
        }));
    });
    println!("done");
}
