use std::collections::LinkedList;
use std::thread;
use std::time::Duration;

use crate::actor::*;
use crate::actor_system::*;
use crate::address::*;
use crate::context::*;
use crate::thread_utils::*;

#[derive(Clone)]
enum CustomMessage {
    SpawnMessage(Address<CustomMessage>, u32),
    StringMessage(String),
}

struct SpawningActor {
    context: Option<Context<CustomMessage>>,
}
impl SpawningActor {}
impl Actor<CustomMessage> for SpawningActor {
    fn handle(&mut self, message: CustomMessage, origin_address: Option<Address<CustomMessage>>) {
        match message {
            CustomMessage::SpawnMessage(address, count) => {
                let ctx: &Context<CustomMessage> = self.context.as_ref().expect("unwrapping context");
                let child_addr = ctx.register_actor(SpawningActor { context: None });
                ctx.send(
                    &address,
                    CustomMessage::StringMessage(format!("Hello #{}", count).to_owned()),
                );
                thread::sleep(Duration::from_millis(1000));
                let count2 = count + 1;
                ctx.send(
                    &child_addr,
                    CustomMessage::SpawnMessage(address.clone(), count2),
                );
            }
            _ => {}
        };
    }

    fn start(&mut self, context: Context<CustomMessage>) {
        self.context = Some(context);
        println!("new SpawningActor");
    }
}

struct OutputActor {}
impl Actor<CustomMessage> for OutputActor {
    fn handle(&mut self, message: CustomMessage, _origin_address: Option<Address<CustomMessage>>) {
        match message {
            CustomMessage::StringMessage(content) => {
                println!("OutputActor received: {}", content);
            }
            _ => {}
        };
    }

    fn start(&mut self, context: Context<CustomMessage>) {}
}

// messages and spawning child actors

pub fn run() {
    println!("init");
    ActorSystem::start(|| {
        let output_actor = ActorSystem::register_actor(OutputActor {}, None);
        let spawning_addr = ActorSystem::register_actor(SpawningActor { context: None }, None);
        spawning_addr.senda(CustomMessage::SpawnMessage(output_actor, 0))
    });
    println!("done");
}
