use std::sync::mpsc::{channel, Sender};

type Message = &'static str;
type Address = Sender<Envelope>;

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
    address: Sender<Envelope>
}

impl Actor for RootActor {

    fn get_context(&mut self) -> &mut ActorContext {
        &mut self.context
    }

    fn handle(&mut self, envelope: Envelope) {
        println!("Initialized root actor");
        //println!("Message: {} from: {}", envelope.message, envelope.sender_address);  
    }

}

pub struct RunSystem {

    root_actor: RootActor

}

pub fn start_system<F>(f: F) -> RunSystem 
    where F: FnOnce() + 'static {

    // TODO: save mailbox of root actor somewhere
    let (root_actor_address, root_actor_mailbox) = channel();
    let mut run_system = RunSystem {
        root_actor : RootActor { 
            context: ActorContext::new(),
            address: root_actor_address
        }
    };
    run_system.init(f);
    run_system

}

impl RunSystem {

    fn init<F>(&mut self, f: F) where 
        F: FnOnce() + 'static
    {
        let (run_system_address, run_system_mailbox) = channel();
        Self::register(&mut self.root_actor, run_system_address.clone());
        let envelope = Envelope {
            message: "Get the party started",
            sender_address: run_system_address
        };
        self.root_actor.handle(envelope);
        f();
    }

    fn register<A>(actor: &mut A, parent_address: Address) where A: Actor + 'static {
        let mut context = actor.get_context();
        context.parent_address = Some(parent_address);
    }

    pub fn spawn<A>(&mut self, actor: &mut A) where A: Actor + 'static {
        Self::register(actor, self.root_actor.address.clone());
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
    let mut system = start_system(|| {
        println!("Started");
    });
    let mut my_actor = MyActor{ context: ActorContext::new() };
    system.spawn(&mut my_actor);
    let (outside_world_address, outside_world_mailbox) = channel();
    my_actor.handle(Envelope { message: "Test", sender_address: outside_world_address });
}
