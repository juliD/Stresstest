extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::prelude::*;

type ActorId = u64;

#[derive(Clone)]
struct Message {
    payload: String,
}

#[derive(Clone)]
struct Envelope {
    message: Message,
}

#[derive(Clone)]
struct ActorAddress {
    id: ActorId,
    sender: Sender<Envelope>,
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
        self.sender
            .clone()
            .send_all(stream::once(Ok(Envelope { message: message })))
            .wait()
            .ok();
    }
}

trait Actor {
    fn handle(&mut self, message: Message, context: Context);
}

#[derive(Clone)]
struct Context {
    system: Arc<Mutex<Router>>,
}
impl Context {
    fn register_actor(&self, actor: Arc<Mutex<Actor + Send>>) -> ActorAddress {
        self.system
            .lock()
            .unwrap()
            .register_actor(actor, self.clone())
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn run_blocking<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        tokio::run(future::ok(()).map(move |_| {
            f();
        }));
    }

    fn spawn<F>(&self, receiver: Receiver<Envelope>, f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        tokio::spawn(receiver.map(f).collect().then(|_| Ok(())));
    }
}

struct Router {
    dispatcher: Dispatcher,
    actors: HashMap<ActorId, Arc<Mutex<Actor + Send>>>,
    actor_id_counter: ActorId,
}
impl Router {
    fn create() -> Router {
        println!("creating ActorSystem");
        Router {
            dispatcher: Dispatcher {},
            actors: HashMap::new(),
            actor_id_counter: 0,
        }
    }

    fn start<F>(f: F)
    where
        F: FnOnce(Context) + 'static + Send,
    {
        // init actor system
        let actor_system = Arc::new(Mutex::new(Router::create()));
        let actor_system_clone = actor_system.clone();

        let context = Context {
            system: actor_system.clone(),
        };
        let context_clone = context.clone();

        Dispatcher::run_blocking(move || {
            println!("setting up system");
            f(context_clone);
        });

        // run blocking stream to keep system alive
        let (_, system_receiver) = channel::<u8>(64);
        tokio::run(system_receiver.map(move |_| {}).collect().then(|_| Ok(())));
    }
    fn register_actor(
        &mut self,
        actor: Arc<Mutex<Actor + Send>>,
        context: Context,
    ) -> ActorAddress {
        self.actor_id_counter += 1;
        self.actors
            .insert(self.actor_id_counter.clone(), actor.clone());

        let (sender, receiver) = channel::<Envelope>(16);

        let address = ActorAddress {
            id: self.actor_id_counter.clone(),
            sender: sender,
        };

        self.dispatcher.spawn(receiver, move |envelope| {
            actor
                .lock()
                .unwrap()
                .handle(envelope.message, context.clone());
        });
        address
    }
}

struct SomeActor {}
impl Actor for SomeActor {
    fn handle(&mut self, message: Message, context: Context) {
        println!("some actor received a message: {}", message.payload);
    }
}

struct OtherActor {
    target: ActorAddress,
}
impl Actor for OtherActor {
    fn handle(&mut self, message: Message, context: Context) {
        println!("other actor received a message: {}", message.payload);
        self.target.send(message);
    }
}

fn main() {
    println!("init");
    Router::start(|context: Context| {
        let some_actor = Arc::new(Mutex::new(SomeActor {}));
        let some_addr = context.register_actor(some_actor);

        let other_actor = Arc::new(Mutex::new(OtherActor {
            target: some_addr.clone(),
        }));
        let other_addr = context.register_actor(other_actor);

        tokio::spawn(future::ok(()).map(move |_| {
            thread::sleep(Duration::from_millis(1000));
            other_addr.send(Message {
                payload: "Hi".to_owned(),
            });
            thread::sleep(Duration::from_millis(2000));
            other_addr.send(Message {
                payload: "Hello".to_owned(),
            });
        }));
    });
    println!("done");
}
