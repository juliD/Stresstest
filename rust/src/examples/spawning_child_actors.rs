extern crate actor_model;
extern crate futures;
extern crate tokio;

use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

// TODO: Just import actor_model::*?
use actor_model::actor::*;
use actor_model::context::*;
use actor_model::tokio_util::*;
use actor_model::router::*;
use actor_model::address::*;

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
    fn handle(&mut self, message: String) {
        println!("SpawningActor received a message: {}", message);

        if message == "Spawn" {
            self.child_id_counter += 1;
            let ctx: &Context = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(ChildActor {
                id: self.child_id_counter,
                context: None,
            });
            self.children.push_front(child_addr.clone());
            child_addr.send("Welcome".to_owned());
            self.children.iter().for_each(|child| {
                child.send("A new sibling arrived".to_owned())
            });
        }
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

struct ChildActor {
    id: u32,
    context: Option<Context>,
}
impl Actor for ChildActor {
    fn handle(&mut self, message: String) {
        println!(
            "ChildActor #{} received message: {}",
            self.id, message
        );

        if message == "A new sibling arrived" {
            let ctx: &Context = self.context.as_ref().expect("");
            let paddr: &Address = ctx.parent_address.as_ref().expect("");
            paddr.send("Wooohoooo".to_owned())
        }
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

struct ForwardingActor {
    target: Address,
}
impl Actor for ForwardingActor {
    fn handle(&mut self, message: String) {
        println!("ForwardingActor received a message: {}", message);
        self.target.send(message);
    }

    fn receive_context(&mut self, _context: Context) {}
}

// messages and spawning child actors

pub fn run() {
    println!("init");
    Router::start(|| {
        let spawning_addr = Router::register_actor(SpawningActor::new(), None);

        let forwarding_addr = Router::register_actor(
            ForwardingActor {
                target: spawning_addr.clone(),
            },
            None,
        );

        TokioUtil::run_background(move || {
            thread::sleep(Duration::from_millis(1000));
            println!("");
            forwarding_addr.send("Spawn".to_owned());
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned());
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned());
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned());
        });
    });
    println!("done");
}
