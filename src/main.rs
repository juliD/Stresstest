type Message = &'static str;
type Address = &'static str;

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
    context: ActorContext
}

impl Actor for RootActor {

    fn get_context(&mut self) -> &mut ActorContext {
        &mut self.context
    }

    fn handle(&mut self, envelope: Envelope) {
        println!("Initialized root actor");
        println!("Message: {} from: {}", envelope.message, envelope.sender_address);  
    }

}

struct RunSystem {}

impl RunSystem {

    pub fn start<F>(f: F) where 
        F: FnOnce() + 'static
    {
        let mut actor = RootActor { context: ActorContext::new()};
        Self::register(&mut actor, "$SYS");
        let envelope = Envelope {
            message: "Get the party started",
            sender_address: "$SYS"
        };
        actor.handle(envelope);
        f();
    }

    fn register<A>(actor: &mut A, parent_address: Address) where A: Actor + 'static {
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
        println!("Message: {} from: {}", envelope.message, envelope.sender_address)
    }

}

fn main() {
    let mut system = RunSystem::start(|| {
        println!("Started");
    });
    let my_actor = MyActor{ context: ActorContext::new() };
}
