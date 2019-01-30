extern crate actor_model;
extern crate futures;
extern crate tokio;

use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::tokio_util::*;

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
    fn handle(&mut self, message: String, origin_address: Option<Address>) {
        println!("SpawningActor received a message: {}", message);

        if message == "Spawn" {
            let ctx: &Context = self.context.as_ref().expect("");

            if let Some(addr) = origin_address {
                // println!("SpawningActor received an Ok");
                ctx.send(&addr, "Ok".to_owned());
            }

            self.child_id_counter += 1;
            let child_addr = ctx.register_actor(ChildActor {
                id: self.child_id_counter,
                context: None,
            });
            self.children.push_front(child_addr.clone());
            ctx.send(&child_addr, "Welcome".to_owned());
            self.children
                .iter()
                .for_each(|child| ctx.send(child, "A new sibling arrived".to_owned()));
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
    fn handle(&mut self, message: String, _sender: Option<Address>) {
        println!("ChildActor #{} received message: {}", self.id, message);

        if message == "A new sibling arrived" {
            let ctx: &Context = self.context.as_ref().expect("");
            let paddr: &Address = ctx.parent_address.as_ref().expect("");
            ctx.send(paddr, "Wooohoooo".to_owned())
        }
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

struct ForwardingActor {
    target: Address,
    context: Option<Context>,
}
impl Actor for ForwardingActor {
    fn handle(&mut self, message: String, _sender: Option<Address>) {
        let ctx: &Context = self.context.as_ref().expect("");
        match message.as_ref() {
            "Ok" => println!("ForwardingActor got a confirmation"),
            _ => println!("ForwardingActor forwarding: {}", message),
        };
        ctx.send(&self.target, message);
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

// messages and spawning child actors

pub fn run() {
    println!("init");
    ActorSystem::start(|| {
        let spawning_addr = ActorSystem::register_actor(SpawningActor::new(), None);

        let forwarding_addr = ActorSystem::register_actor(
            ForwardingActor {
                target: spawning_addr.clone(),
                context: None,
            },
            None,
        );

        TokioUtil::run_background(move || {
            thread::sleep(Duration::from_millis(1000));
            println!("");
            forwarding_addr.send("Spawn".to_owned(), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned(), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned(), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            forwarding_addr.send("Spawn".to_owned(), None);
        });
    });
    println!("done");
}
