extern crate futures;
extern crate tokio;

use futures::future::lazy;
use futures::sync::mpsc::{channel, Sender};
use futures::{stream, Future, Sink, Stream};

type Message = &'static str;
type Address = Sender<Envelope>;
// TODO: put into RunSystem config
const MAILBOX_SIZE: usize = 1_024;

#[derive(Debug)]
pub struct Envelope {
    message: Message,
    sender_address: Option<Address>,
}

#[derive(Default)]
pub struct ActorContext {
    parent_address: Option<Address>,
}

impl ActorContext {
    fn new() -> ActorContext {
        Default::default()
    }
}

pub trait Actor {
    fn add_parent_address(&mut self, parent_address: Address) {
        let mut context = self.get_context();
        context.parent_address = Some(parent_address);
    }

    // TODO: find nicer structure so that no user needs to implement get_context
    fn get_context(&mut self) -> &mut ActorContext;

    fn handle(&mut self, envelope: Envelope);
}

struct RootActor {
    context: ActorContext,
    address: Address,
    children: Vec<Box<dyn Actor + Send>>,
}

impl Actor for RootActor {
    fn get_context(&mut self) -> &mut ActorContext {
        &mut self.context
    }

    fn handle(&mut self, envelope: Envelope) {
        println!("Initialized root actor");
    }
}

pub struct RunSystem {
    root_actor: RootActor,
}

impl RunSystem {
    /*   pub fn new() -> RunSystem {

           // TODO: save mailbox of root actor somewhere
           let (root_actor_address, root_actor_mailbox) = channel(MAILBOX_SIZE);
           let mut run_system = RunSystem {
               root_actor : RootActor {
                   context: ActorContext::new(),
                   address: root_actor_address,
                   children: vec![]
               }
           };
           run_system.init();
           run_system

       }

       fn init(&mut self) {
           let (run_system_address, run_system_mailbox) = channel(MAILBOX_SIZE);
           Self::register(&mut self.root_actor, run_system_address.clone());
           let envelope = Envelope {
               message: "Get the party started",
               sender_address: run_system_address
           };
           self.root_actor.handle(envelope);
       }
    */
    /* pub fn start<T>(f: T) where
        T: FnOnce(Address) -> Box<Future<Item=(),Error=()>> + Send + 'static
    {
        let (root_actor_address, root_actor_mailbox) = channel(MAILBOX_SIZE);
        // TODO: Why is the lazy needed?
        tokio::run(lazy(move || {
            tokio::spawn(root_actor_mailbox.for_each(|msg|{
                println!("got message");
                Ok(())
            }));
            f(root_actor_address);
            Ok(())
        }));
    } */
    /* pub fn start<T>(f: T) where
        T: FnOnce(Address) + Send + 'static
    {
        let (root_actor_address, root_actor_mailbox) = channel(MAILBOX_SIZE);
        // TODO: Why is the lazy needed?
        tokio::run(lazy(move || {
            tokio::spawn(root_actor_mailbox.for_each(|msg|{
                println!("got message");
                Ok(())
            }));
            f(root_actor_address);
            Ok(())
        }));
    } */
    pub fn start<T>(mut user_root_actor: T)
    where
        T: Actor + Send + 'static,
    {
        let (root_actor_address, root_actor_mailbox) = channel(MAILBOX_SIZE);
        let (user_root_actor_address, user_root_actor_mailbox) = channel(MAILBOX_SIZE);
        // TODO: Why is the lazy needed?
        tokio::run(lazy(move || {
            // spawn the system's root actor
            tokio::spawn(root_actor_mailbox.for_each(|msg: Envelope| {
                println!("root got message");
                Ok(())
            }));
            // spawn the user's root actor
            tokio::spawn(user_root_actor_mailbox.for_each(move |msg: Envelope| {
                user_root_actor.handle(msg);
                Ok(())
            }));
            // send the system start message to the user actor
            user_root_actor_address
                .send(Envelope {
                    message: "hallo",
                    sender_address: Some(root_actor_address),
                })
                .map_err(|e| println!("socket error = {:?}", e))
                .and_then(|_| Ok(()))
        }));
    }

    /*     fn register<A>(actor: &mut A, parent_address: Address) where A: Actor {
        let mut context = actor.get_context();
        context.parent_address = Some(parent_address);
    }

    pub fn register_root<A: 'static>(&mut self, mut actor: A) where A: Actor {
        let mut context = actor.get_context();
        context.parent_address = Some(self.root_actor.address.clone());
        self.root_actor.children.push(Box::new(actor));
    } */
}

struct MyActor {
    context: ActorContext,
}

impl Actor for MyActor {
    fn get_context(&mut self) -> &mut ActorContext {
        &mut self.context
    }

    fn handle(&mut self, envelope: Envelope) {
        println!("Message: {}", envelope.message);
    }
}

fn main() {
    //let mut system = RunSystem::new();
    let my_actor = MyActor {
        context: ActorContext::new(),
    };
    // system.register_root(my_actor);
    RunSystem::start(my_actor);
}
