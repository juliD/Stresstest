extern crate actor_model;
extern crate futures;
extern crate tokio;

use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

use actor_model::actor::*;
use actor_model::context::*;
use actor_model::thread_utils::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;

type CustomMessage = String;

struct SpawningActor {
    children: LinkedList<Address<CustomMessage>>,
    child_id_counter: u32,
    context: Option<Context<CustomMessage>>,
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
impl Actor<CustomMessage> for SpawningActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<CustomMessage>>) {
        println!("SpawningActor received a message: {}", message);

        if message == "Spawn" {
            let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");

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

    fn receive_context(&mut self, context: Context<CustomMessage>) {
        self.context = Some(context);
    }
}

struct ChildActor {
    id: u32,
    context: Option<Context<CustomMessage>>,
}
impl Actor<CustomMessage> for ChildActor {
    fn handle(&mut self, message: String, _origin_address: Option<Address<CustomMessage>>) {
        println!("ChildActor #{} received message: {}", self.id, message);

        if message == "A new sibling arrived" {
            let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");
            let paddr: &Address<CustomMessage> = ctx.parent_address.as_ref().expect("");
            ctx.send(paddr, "Wooohoooo".to_owned())
        }
    }

    fn receive_context(&mut self, context: Context<CustomMessage>) {
        self.context = Some(context);
    }
}

struct ForwardingActor {
    target: Address<CustomMessage>,
    context: Option<Context<CustomMessage>>,
}
impl Actor<CustomMessage> for ForwardingActor {
    fn handle(&mut self, message: CustomMessage, _origin_address: Option<Address<CustomMessage>>) {
        let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");
        match message.as_ref() {
            "Ok" => println!("ForwardingActor got a confirmation"),
            _ => println!("ForwardingActor forwarding: {}", message),
        };
        ctx.send(&self.target, message);
    }

    fn receive_context(&mut self, context: Context<CustomMessage>) {
        self.context = Some(context);
    }
}

// messages and spawning child actors

pub fn run() {
    println!("init");
    ActorSystem::<CustomMessage>::start(|| {
        let spawning_addr = ActorSystem::register_actor(SpawningActor::new(), None);
        println!("hallo");
        let forwarding_addr = ActorSystem::register_actor(
            ForwardingActor {
                target: spawning_addr.clone(),
                context: None,
            },
            None,
        );

        ThreadUtils::<CustomMessage>::run_background(move || {
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
