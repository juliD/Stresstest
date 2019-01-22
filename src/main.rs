extern crate tokio;
extern crate futures;

use futures::{stream, Future, Stream, Sink, Async};
use futures::future::lazy;
use futures::sync::mpsc::{Receiver, Sender, channel};

type Message = &'static str;
type Address = Sender<Envelope>;
type Mailbox = Receiver<Envelope>;
// TODO: put into RunSystem config
const MAILBOX_SIZE: usize = 1_024;

pub struct Envelope {
    message: Message,
    sender_address: Address
}

#[derive(Default)]
pub struct ActorContext {
    parent_address: Option<Address>,
    mailbox: Option<Mailbox>
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
    address: Address
}

impl Actor for RootActor {

    fn get_context(&mut self) -> &mut ActorContext {
        &mut self.context
    }

    fn handle(&mut self, envelope: Envelope) {
        println!("Calling handle of root actor");
        println!("RootActor got envelope with message {}", envelope.message);
    }

}

pub struct RunSystem {

    root_actor: RootActor,
    run_system_address: Sender<Envelope>

}


impl RunSystem {

    pub fn new() -> RunSystem {
        println!("Initializing RunSystem");
        let root_actor = Self::init_root_actor();
        let (run_system_address, _run_system_mailbox) = channel(MAILBOX_SIZE);
        let mut run_system = RunSystem {
            root_actor: root_actor,
            run_system_address: run_system_address
        };
        run_system.register_root_actor();
        run_system
    }

    fn init_root_actor() -> RootActor {
        println!("Initializing RootActor");
        // TODO: save root_actor_mailbox to root actor
        let (root_actor_address, root_actor_mailbox) = channel(MAILBOX_SIZE);
        let mut root_actor = RootActor { 
            context: ActorContext::new(),
            address: root_actor_address
        };
        root_actor.context.mailbox = Some(root_actor_mailbox);
        root_actor
    }

    fn register_root_actor(&mut self) {
        println!("Registering RootActor");
        Self::register(&mut self.root_actor, self.run_system_address.clone());
    }

    pub fn start<F>(self, f: F) where F: FnOnce() + Send + 'static {
        println!("Starting system");
        tokio::run(lazy(move|| {
            f();
            let envelope = Envelope {
                message: "Get the party started",
                sender_address: self.run_system_address
            };
            Self::send_envelope_async(envelope, self.root_actor.address, self.root_actor.context.mailbox.unwrap());
            Ok(())
        }));
    }

    fn send_envelope_async(envelope: Envelope, address: Address, mut mailbox: Mailbox) {
        tokio::spawn(lazy(move|| {
            address.send(envelope).wait().expect("Some error while sending StartMessage");
                
            while let Ok(Async::Ready(Some(v))) = mailbox.poll() {
                println!("stream: {}", v.message);
            }
            Ok(())
        }));
    }

    fn register<A>(actor: &mut A, parent_address: Address) where A: Actor {
        let mut context = actor.get_context();
        context.parent_address = Some(parent_address);
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

impl MyActor {
    pub fn test(self) {
        println!("Test function");
    }
}

fn main() {
    let system = RunSystem::new();
    let my_actor = MyActor{ context: ActorContext::new() };
    system.start(move|| {
        println!("Inside RunSystem user-defined closure");
        my_actor.test();
    });
}
