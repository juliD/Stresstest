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
use actor_model::thread_utils::*;

#[derive(Clone)]
enum CustomMessage {
    SpawnMessage(u32),
    StringMessage(String),
}

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

    fn spawn_child(&mut self) {
        let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");
        self.child_id_counter += 1;
        let child_addr = ctx.register_actor(ChildActor {
            id: self.child_id_counter,
            context: None,
        });
        self.children.push_front(child_addr.clone());
        ctx.send(
            &child_addr,
            CustomMessage::StringMessage("Welcome".to_owned()),
        );
    }
}
impl Actor<CustomMessage> for SpawningActor {
    fn handle(&mut self, message: CustomMessage, origin_address: Option<Address<CustomMessage>>) {
        match message {
            CustomMessage::StringMessage(content) => {
                println!("SpawningActor received a message: {}", content);
            }
            CustomMessage::SpawnMessage(count) => {
                println!("SpawningActor spawning {} children", count);

                for _i in 0..count {
                    self.spawn_child();
                }

                let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");
                self.children.iter().for_each(|child| {
                    ctx.send(
                        child,
                        CustomMessage::StringMessage("A new sibling arrived".to_owned()),
                    )
                });

                if let Some(addr) = origin_address {
                    // println!("SpawningActor received an Ok");
                    ctx.send(&addr, CustomMessage::StringMessage("Ok".to_owned()));
                }
            }
        };
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
    fn handle(&mut self, message: CustomMessage, _origin_address: Option<Address<CustomMessage>>) {
        match message {
            CustomMessage::StringMessage(content) => {
                if content == "A new sibling arrived" {
                    let ctx: &Context<CustomMessage> = self.context.as_ref().expect("");
                    let paddr: &Address<CustomMessage> = ctx.parent_address.as_ref().expect("");
                    ctx.send(paddr, CustomMessage::StringMessage("Wooohoooo".to_owned()));
                }
            }
            _ => {}
        };
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

        ThreadUtils::<CustomMessage>::run_background(move || {
            thread::sleep(Duration::from_millis(1000));
            println!("");
            spawning_addr.send(CustomMessage::SpawnMessage(2), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            spawning_addr.send(CustomMessage::SpawnMessage(1), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            spawning_addr.send(CustomMessage::SpawnMessage(3), None);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            spawning_addr.send(CustomMessage::SpawnMessage(1), None);
        });
    });
    println!("done");
}
