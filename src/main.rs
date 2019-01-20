extern crate tokio;
extern crate futures;

use futures::{stream, Future, Stream, Sink};
use futures::future::lazy;
use futures::sync::mpsc::{Sender, channel};

type Message = &'static str;
type Address = Sender<Envelope>;
// TODO: put into RunSystem config
const MAILBOX_SIZE: usize = 1_024;

pub struct Envelope {
    message: Message,
    sender_address: Address
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
    children: Vec<&Actor>
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

    root_actor: RootActor

}


impl RunSystem {

    pub fn new() -> RunSystem {

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

    pub fn start(& self) {
        tokio::run(lazy(|| {





            Ok(())
        }));
    }

    fn register<A>(actor: &mut A, parent_address: Address) where A: Actor {
        let mut context = actor.get_context();
        context.parent_address = Some(parent_address);
    }

    pub fn register_root<a', A>(&'a self, actor: &'a mut A) where A: Actor {
        let mut context = actor.get_context();
        context.parent_address = Some(self.root_actor.address.clone());
        self.root_actor.children.push(actor);
    }

}

struct MyActor {
    context: ActorContext
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
    let system = RunSystem::new();
    let mut my_actor = MyActor{ context: ActorContext::new() };
    system.register_root(&mut my_actor);
    system.start();
}
