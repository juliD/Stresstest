extern crate actor_model;
extern crate futures;
extern crate tokio;

use std::any::{Any, TypeId};
use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::message::*;
use actor_model::tokio_util::*;

struct StringMessage {
    content: String,
}
impl Message for StringMessage {}

struct SpawnMessage {
    count: u32,
}
impl Message for SpawnMessage {}

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

    fn spawn_child(&mut self) {
        let ctx: &Context = self.context.as_ref().expect("");

        self.child_id_counter += 1;
        let child_addr = ctx.register_actor(ChildActor {
            id: self.child_id_counter,
            context: None,
        });
        self.children.push_front(child_addr.clone());
        ctx.send(
            &child_addr,
            StringMessage {
                content: "Welcome".to_owned(),
            },
        );
    }
}
impl Actor for SpawningActor {
    fn handle(&mut self, message: Box<Message>, origin_address: Option<Address>) {
        let message1: Option<Box<Message>> = match message.downcast::<SpawnMessage>() {
            Ok(spawn_msg) => {
                let count = spawn_msg.count;
                println!("SpawningActor spawning {} children", count);

                self.spawn_child();

                let ctx: &Context = self.context.as_ref().expect("");

                self.children.iter().for_each(|child| {
                    ctx.send(
                        &child,
                        StringMessage {
                            content: "A new sibling arrived".to_owned(),
                        },
                    )
                });

                if let Some(addr) = origin_address {
                    ctx.send(
                        &addr,
                        StringMessage {
                            content: "Ok".to_owned(),
                        },
                    );
                }
                None
            }
            Err(msg) => Some(msg),
        };

        if let Some(_) = message1 {
            println!("SpawningActor received unknown Message");
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
    fn handle(&mut self, message: Box<Message>, _origin_address: Option<Address>) {
        if let Ok(string_msg) = message.downcast::<StringMessage>() {
            let content = string_msg.content;
            println!("ChildActor #{} received message: {}", self.id, content);

            if content == "A new sibling arrived" {
                let ctx: &Context = self.context.as_ref().expect("");
                let paddr: &Address = ctx.parent_address.as_ref().expect("");
                ctx.send(
                    paddr,
                    StringMessage {
                        content: "Wooohoooo".to_owned(),
                    },
                );
            }
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
    fn handle(&mut self, message: Box<Message>, _origin_address: Option<Address>) {
        let ctx: &Context = self.context.as_ref().expect("");

        let message1: Option<Box<Message>> = match message.downcast::<StringMessage>() {
            Ok(string_msg) => {
                let content = string_msg.content;
                match content.as_ref() {
                    "Ok" => println!("ForwardingActor got a confirmation"),
                    _ => {
                        println!("ForwardingActor forwarding StringMessage: {}", content);
                        ctx.send(&self.target, StringMessage { content: content });
                    }
                };
                None
            }
            Err(msg) => Some(msg),
        };

        let message2: Option<Box<Message>> = apply(message1, |msg| {
            let r: Option<Box<Message>> = match msg.downcast::<SpawnMessage>() {
                Ok(spawn_msg) => {
                    let count = spawn_msg.count;
                    println!("ForwardingActor forwarding SpawnMessage");
                    ctx.send(&self.target, SpawnMessage { count: count });
                    None
                }
                Err(msg) => Some(msg),
            };
            r
        });
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
    }
}

fn apply<F>(message: Option<Box<Message>>, f: F) -> Option<Box<Message>>
where
    F: Fn(Box<Message>) -> Option<Box<Message>>,
{
    if (message.is_some()) {
        return f(message.unwrap());
    }
    None
}

fn is<T: ?Sized + Any>(_s: &T, type_id: TypeId) -> bool where {
    TypeId::of::<T>() == type_id
}

fn is_spawnmessage<T: ?Sized + Any>(_s: &T) -> bool where {
    TypeId::of::<T>() == TypeId::of::<SpawnMessage>()
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
            send_spawn_command(&forwarding_addr);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            send_spawn_command(&forwarding_addr);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            send_spawn_command(&forwarding_addr);
            thread::sleep(Duration::from_millis(2000));
            println!("");
            send_spawn_command(&forwarding_addr);
        });
    });
    println!("done");
}

fn send_spawn_command(addr: &Address) {
    addr.send(SpawnMessage { count: 1 }, None);
}
