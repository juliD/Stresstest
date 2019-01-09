extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use tokio::prelude::task::Task;
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
        self.sender.clone().send_all(stream::once(Ok(message))).wait().ok();
    }
}

trait Actor {}

// struct Mailbox {
//     actor_id_counter: ActorId,
//     // receiver: Receiver<Message>,
//     task: Arc<Mutex<Option<Task>>>,
// }
// impl Mailbox {
//     fn set_task(&mut self, task: Task) {
//         let mut guard = self.task.lock().unwrap();
//         *guard = Some(task);
//     }
// }

struct Router {
    actors: HashMap<ActorId, Arc<Mutex<RootActor>>>,
    // mailboxes: HashMap<ActorId, Arc<Mutex<Mailbox>>>,
    actor_id_counter: ActorId,
}
impl Router {
    fn new() -> Router {
        Router {
            actors: HashMap::new(),
            // mailboxes: HashMap::new(),
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

struct SystemMessageRegisterActor {
    actor: Arc<Mutex<RootActor>>,
}

#[derive(Clone)]
struct Context {
    sender_register_actor: Sender<SystemMessageRegisterActor>,
    system: Arc<Mutex<ActorSystem>>,
}
impl Context {
    // fn register_actor(&self, actor: Arc<Mutex<RootActor>>) {
    //     println!("registering actor");
    //     self.sender_register_actor
    //         .clone()
    //         .send_all(stream::once(Ok(SystemMessageRegisterActor {
    //             actor: actor,
    //         })))
    //         .wait()
    //         .ok();
    // }

    fn register_actor(&self, actor: Arc<Mutex<RootActor>>) -> ActorAddress {
        self.system.lock().unwrap().register_actor(actor)
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn start_main() {
        tokio::run(future::ok(()).map(move |_| {
            println!("creating ActorSystem");

            let (system_sender, system_receiver) = channel::<SystemMessageRegisterActor>(64);

            let mut actor_system = Arc::new(Mutex::new(ActorSystem::create(system_sender.clone())));
            let context = Context {
                sender_register_actor: system_sender.clone(),
                system: actor_system.clone(),
            };

            let actor = Arc::new(Mutex::new(RootActor {}));
            let address = context.register_actor(actor);
            address.send("first actor message ever sent".to_string());

            tokio::spawn(
                system_receiver
                    .map(move |message: SystemMessageRegisterActor| {
                        println!("ActorSystem received SystemMessageRegisterActor");
                        actor_system.lock().unwrap().handle(message);
                    })
                    .collect()
                    .then(|_| Ok(())),
            );

            loop {}
        }));
    }

    fn register_mailbox(&self, receiver: Receiver<Message>) {
        println!("spawning task: mailbox");
        tokio::spawn(
            receiver
                .map(|message: Message| println!("message {}", message))
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
    fn create(system_sender: Sender<SystemMessageRegisterActor>) -> ActorSystem {
        ActorSystem {
            dispatcher: Dispatcher {},
            router: Router::new(),
        }
    }

    fn start() {
        Dispatcher::start_main();
    }
    fn register_actor(&mut self, actor: Arc<Mutex<RootActor>>) -> ActorAddress {
        let (address, mailbox_receiver) = self.router.register_actor(actor);
        self.dispatcher.register_mailbox(mailbox_receiver);
        address
    }
}
impl Handler<SystemMessageRegisterActor> for ActorSystem {
    fn handle(&mut self, message: SystemMessageRegisterActor) {
        self.register_actor(message.actor);
    }
}

trait Handler<T> {
    fn handle(&mut self, messae: T);
}

struct RootActor;
impl Actor for RootActor {}
impl Handler<Message> for RootActor {
    fn handle(&mut self, message: Message) {
        println!("root actor received message: {}", message);
    }
}

fn main() {
    println!("init");
    ActorSystem::start();
    println!("done");
}
