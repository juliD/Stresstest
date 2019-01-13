extern crate futures;
extern crate tokio;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod message;
mod dispatcher;
mod actor;
mod router;

use crate::message::*;
use crate::dispatcher::*;
use crate::actor::*;
use crate::router::*;


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

        Dispatcher::run_background(move || {
            thread::sleep(Duration::from_millis(1000));
            other_addr.send(Message {
                payload: "Hi".to_owned(),
            });
            thread::sleep(Duration::from_millis(2000));
            other_addr.send(Message {
                payload: "Hello".to_owned(),
            });
        });
    });
    println!("done");
}
