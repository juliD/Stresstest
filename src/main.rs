extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tokio::prelude::task::Task;
use tokio::prelude::*;

// struct CheckMailboxTask {
//     mailbox: Arc<Mutex<Mailbox>>,
// }
// impl CheckMailboxTask {
//     fn run() {
//         println!("running task");
//     }
// }
// impl Future for CheckMailboxTask {
//     type Item = ();
//     type Error = ();

//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         println!("poll waits for lock of mailbox");
//         let mut mailbox = self.mailbox.lock().unwrap();
//         println!("poll enters lock of mailbox");
//         let task = futures::task::current();
//         mailbox.set_task(task);
//         let message: Result<Message, TryRecvError> = mailbox.receiver.try_recv();
//         let r = match message {
//             Ok(msg) => {
//                 println!("polled ready");
//                 Ok(Async::Ready(()))
//             }
//             _ => {
//                 println!("polled not ready");
//                 Ok(Async::NotReady)
//             }
//         };
//         println!("poll leaves lock of mailbox");
//         r
//     }
// }

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

trait Actor {}

struct Mailbox {
    actor_id_counter: ActorId,
    // receiver: Receiver<Message>,
    task: Arc<Mutex<Option<Task>>>,
}
impl Mailbox {
    fn set_task(&mut self, task: Task) {
        let mut guard = self.task.lock().unwrap();
        *guard = Some(task);
    }
}

struct Router {
    actors: HashMap<ActorId, TestActor>,
    mailboxes: HashMap<ActorId, Arc<Mutex<Mailbox>>>,
    actor_id_counter: ActorId,
}
impl Router {
    fn new() -> Router {
        Router {
            actors: HashMap::new(),
            mailboxes: HashMap::new(),
            actor_id_counter: 0,
        }
    }

    fn register_actor(&mut self, actor: TestActor) -> (ActorAddress, Receiver<Message>) {
        self.actor_id_counter += 1;
        self.actors.insert(self.actor_id_counter.clone(), actor);

        let (sender, receiver) = channel::<Message>(16);
        let mailbox = Mailbox {
            actor_id_counter: self.actor_id_counter.clone(),
            // receiver: rx,
            task: Arc::new(Mutex::new(Option::None)),
        };

        let address = ActorAddress {
            id: self.actor_id_counter.clone(),
            sender: sender,
        };
        (address, receiver)
    }

    fn get_mailbox(&self, id: &ActorId) -> Option<Arc<Mutex<Mailbox>>> {
        match self.mailboxes.get(id) {
            Option::None => Option::None,
            Option::Some(m) => Option::Some(Arc::clone(m)),
        }
    }
}

struct SystemMessageRegisterActor {
    actor: Arc<Mutex<TestActor>>,
}

#[derive(Clone)]
struct Context {
    sender_register_actor: Sender<SystemMessageRegisterActor>,
}
impl Context {
    fn register_actor(&self, actor: Arc<Mutex<TestActor>>) {
        println!("registering actor");
        self.sender_register_actor
            .clone()
            .send_all(stream::once(Ok(SystemMessageRegisterActor {
                actor: actor,
            })))
            .wait()
            .ok();
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn start_main(
        &self,
        receiver_register_actor: Receiver<SystemMessageRegisterActor>,
        context: Context,
    ) {
        tokio::run(future::ok(()).map(move |_| {
            println!("spawning task: receiver for SystemMessageRegisterActor");
            tokio::spawn(
                receiver_register_actor
                    .map(|message: SystemMessageRegisterActor| {
                        println!("ActorSystem received SystemMessageRegisterActor")
                    })
                    .collect()
                    .then(|_| Ok(())),
            );

            let actor = Arc::new(Mutex::new(TestActor {}));
            context.register_actor(actor);
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
    context: Context,
}
impl ActorSystem {
    fn start() {
        let (sender_register_actor, receiver_register_actor) =
            channel::<SystemMessageRegisterActor>(64);

        let context = Context {
            sender_register_actor: sender_register_actor.clone(),
        };

        let actor_system = ActorSystem {
            dispatcher: Dispatcher {},
            router: Router::new(),
            context,
        };

        actor_system
            .dispatcher
            .start_main(receiver_register_actor, actor_system.context.clone());
    }
    fn register_actor(&mut self, actor: TestActor) -> ActorAddress {
        let (address, mailbox_receiver) = self.router.register_actor(actor);
        self.dispatcher.register_mailbox(mailbox_receiver);
        address
    }
}

struct TestActor;
impl Actor for TestActor {}

fn main() {
    println!("init");
    ActorSystem::start();
    println!("done");
}
