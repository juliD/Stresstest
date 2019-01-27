extern crate actor_model;
extern crate futures;
extern crate serde_json;
extern crate tokio;
extern crate tokio_serde_json;

use futures::{Future, Stream};
use serde_json::Value;
use tokio::{
    codec::{FramedRead, LengthDelimitedCodec},
    net::TcpListener,
};
use tokio_serde_json::ReadJson;

use std::collections::LinkedList;

// TODO: Just import actor_model::*?
use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
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
            self.children
                .iter()
                .for_each(|child| child.send("A new sibling arrived".to_owned()));
        }
    }

    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
        println!("init SpawningActor");
    }
}

struct ChildActor {
    id: u32,
    context: Option<Context>,
}
impl Actor for ChildActor {
    fn handle(&mut self, message: String) {
        println!("ChildActor #{} received message: {}", self.id, message);

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

// messages and spawning child actors

pub fn run() {
    println!("init");
    Router::start(|| {
        let spawning_addr = Router::register_actor(SpawningActor::new(), None);

        let listener = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
        println!("listening on {:?}", listener.local_addr());
        tokio::spawn(
            listener
                .incoming()
                .for_each(move |socket| {

                    let spawning_addr_clone = spawning_addr.clone();

                    // Delimit frames using a length header
                    let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
                    // Deserialize frames
                    let deserialized = ReadJson::<_, Value>::new(length_delimited)
                        .map_err(|e| println!("ERR: {:?}", e));
                    // Spawn a task that prints all received messages to STDOUT
                    tokio::spawn(deserialized.for_each(move |msg| {
                        println!("GOT: {:?}", msg);
                        println!(
                            "{:?}",
                            msg.as_object()
                                .unwrap()
                                .get("root")
                                .unwrap()
                                .as_str()
                                .unwrap()
                        );

                        spawning_addr_clone.send("Spawn".to_owned());

                        Ok(())
                    }));
                    Ok(())
                })
                .map_err(|_| ()),
        );
    });
    println!("done");
}
