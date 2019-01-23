extern crate actor_model;
extern crate futures;
extern crate tokio;

use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

use actor_model::actor::*;
use actor_model::dispatcher::*;
use actor_model::message::*;
use actor_model::router::*;

struct SpawningActor {
    children: LinkedList<Address>,
    child_id_counter: u32,
    context: Option<Context>,
}
impl SpawningActor {
    fn new() -> SpawningActor {
        SpawningActor {
            children: LinkedList::new(),
            child_id_counter: 0,
            context: None,
        }
    }
}
impl Actor for SpawningActor {
    fn handle(&mut self, message: Message) {
        println!("SpawningActor received a message: {}", message.payload);

        if message.payload == "Spawn" {
            self.child_id_counter += 1;
            let ctx = match &self.context {
                Some(c) => c,
                None => panic!("bla")
            };
            let child_addr = ctx.register_actor(ChildActor {
                id: self.child_id_counter,
            }, );
            self.children.push_front(child_addr.clone());
            child_addr.send(Message {
                payload: "Welcome".to_owned(),
            });
            self.children.iter().for_each(|child| {
                child.send(Message {
                    payload: "A new sibling arrived".to_owned(),
                })
            });
        }
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

struct ChildActor {
    id: u32,
}
impl Actor for ChildActor {
    fn handle(&mut self, message: Message) {
        println!(
            "ChildActor #{} received message: {}",
            self.id, message.payload
        );
    }

    fn receive_context(&mut self, context: Context) { }
}

struct ForwardingActor {
    target: Address,
}
impl Actor for ForwardingActor {
    fn handle(&mut self, message: Message) {
        println!("ForwardingActor received a message: {}", message.payload);
        self.target.send(message);
    }

    fn receive_context(&mut self, context: Context) { }
}



// messages and spawning child actors

pub fn run() {
    println!("init");
    Router::start(|| {

        let spawning_addr = Router::register_actor(SpawningActor::new(), None);

        let forwarding_addr = Router::register_actor(ForwardingActor {
            target: spawning_addr.clone(),
        }, None);

        Dispatcher::run_background(move || {

            thread::sleep(Duration::from_millis(1000));
            println!("");
            forwarding_addr.send(Message {
                payload: "Spawn".to_owned(),
            });
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send(Message {
                payload: "Spawn".to_owned(),
            });
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send(Message {
                payload: "Spawn".to_owned(),
            });
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send(Message {
                payload: "Spawn".to_owned(),
            });
        });
    });
    println!("done");
}
