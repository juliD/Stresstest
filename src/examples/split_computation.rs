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

struct ComputingNode {
    results: Vec<u64>,
}
impl ComputingNode {
    fn new() -> ComputingNode {
        ComputingNode {
            results: Vec::new(),
        }
    }
}
impl Actor for ComputingNode {
    fn handle(&mut self, message: Message) {
        println!("ComputingNode received a message: {}", message.payload);

        let mut split = message.payload.splitn(3, " ");
        if split.next().unwrap() == "COMPUTE" {
            let now = std::time::SystemTime::now();
            println!("starting computation...");
            let start = split.next().unwrap().parse::<u64>().unwrap();
            let end = split.next().unwrap().parse::<u64>().unwrap();

            for i in start..end {
                self.results.push(i * i);
            }

            println!(
                "finished computation {} - {} in {} seconds",
                start,
                end,
                now.elapsed().unwrap().as_secs()
            );
        }
    }
}

pub fn run() {
    println!("init");
    Router::start(|| {
        let nodes = vec![
            Router::register_actor(ComputingNode::new()),
            Router::register_actor(ComputingNode::new()),
            Router::register_actor(ComputingNode::new()),
            Router::register_actor(ComputingNode::new()),
            Router::register_actor(ComputingNode::new()),
        ];

        Dispatcher::run_background(move || {
            for i in 0..nodes.len() {
                nodes.get(i).unwrap().send(Message {
                    payload: format!("COMPUTE {} {}", i * 10000000, (i + 1) * 10000000).to_owned(),
                });
            }
        });


//         Dispatcher::run_background(move || {
//             let now = std::time::SystemTime::now();
//             println!("start computation");
//             let mut results = Vec::new();
//             for i in 0..50000000_u64 {
//                 let r = i * i;
//                 results.push(r);
//             }
//             // println!("{}", results.get(999999).unwrap());
//             println!(
//                 "finish computation in {} seconds",
//                 now.elapsed().unwrap().as_secs()
//             );
//         });

    });
    println!("done");
}