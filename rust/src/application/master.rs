extern crate actor_model;
extern crate futures;
extern crate hyper;
extern crate mpi;
extern crate tokio;
use mpi::traits::*;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use std::collections::LinkedList;
use std::time::Duration;

use futures::*;
use futures::*;
use hyper::rt::{self, Future};
use hyper::Client;
use mpi::topology::Rank;
use std::char;
use std::io::{self, BufRead};
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use mpi::request::WaitGuard;
use std::thread::*;
use mpi::Threading;

// #[derive(Clone)]
type Message = String;

struct WorkerActor {
    context: Option<Context<Message>>,
    target: hyper::Uri,
}
impl WorkerActor {}

impl Actor<Message> for WorkerActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();

        match vec[0] {
            "start" => {
                println!("CHILD GOT START");
                let ctx: &Context<Message> = self.context.as_ref().expect("");
                let paddr: &Address<Message> = ctx.parent_address.as_ref().expect("");
                let own_addr: &Address<Message> = &ctx.own_address;
                // for _ in 0..10{

                rt::spawn(rt::lazy(move || {
                    let client = Client::new();
                    let uri = "http://httpbin.org/ip".parse().expect("unwrap 0");
                    client
                        .get(uri)
                        .map(move |res| {
                            println!("HTTP Response {}", res.status());
                        })
                        .map_err(|err| {
                            println!("Error: {}", err);
                        })
                }));
                // paddr.send("1".to_owned());
                // }
                // own_addr.send("start".to_owned());
            }
            "target" => {
                self.target = vec[1].parse().expect("unwrap 1");
            }
            "stop" => {
                println!("Stopped");
            }
            _ => {}
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}

struct LogActor {
    id: u32,
    context: Option<Context<Message>>,
}

impl Actor<Message> for LogActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {}

    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}

struct MasterActor {
    children: LinkedList<Address<Message>>,
    child_id_counter: u32,
    context: Option<Context<Message>>,
}
impl MasterActor {
    fn new() -> MasterActor {
        MasterActor {
            children: LinkedList::new(),
            child_id_counter: 0,
            context: None,
        }
    }
}

fn broadcast_children(list: &LinkedList<Address<Message>>, count: u32, message: String) {
    let mut iter = list.iter();
    iter.next();

    for _ in 0..count - 1 {
        match iter.next() {
            Some(c) => {
                c.send(message.to_owned(), None);
            }
            _ => {}
        }
    }
}

impl Actor<Message> for MasterActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();
        println!("master got {}", message);
        if vec[0] == "input" {
            match vec[1] {
                "start" => {
                    //TODO: send all worker children start message
                    broadcast_children(&self.children, self.child_id_counter, "start".to_string());
                }
                "stop" => {
                    broadcast_children(&self.children, self.child_id_counter, "stop".to_string());
                }
                "log" => {}
                "target" => {
                    //TODO:check if target correct
                    broadcast_children(&self.children, self.child_id_counter, message);
                }
                _ => println!("Not a valid input. Write help to get a list of commands."),
            }
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        println!("init MasterActor");

        //TODO: LogActor
        // self.child_id_counter += 1;
        // let ctx: &Context = self.context.as_ref().expect("");
        // let child_addr = ctx.register_actor(LogActor {
        //     id: self.child_id_counter,
        //     context: None,
        // });
        // self.children.push_front(child_addr.clone());

        //end if master process

        //create as many actors as cores available
        let num = 2;
        for x in 1..num {
            self.child_id_counter += 1;
            let ctx: &Context<Message> = self.context.as_ref().expect("");

            let child_addr = ctx.register_actor(WorkerActor {
                context: None,
                target: "http://httpbin.org/ip".parse().expect("unwrap 2"),
            });

            self.children.push_back(child_addr.clone());
        }
    }
}

struct InputActor {
    context: Option<Context<Message>>,
}
impl Actor<Message> for InputActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        if message == "watch input" {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(input) => {
                        let ctx: &Context<Message> = self.context.as_ref().expect("");
                        let paddr: &Address<Message> = ctx.parent_address.as_ref().expect("");
                        let answer = format!("broadcast {}", input);
                        match input.as_ref() {
                            "help" => {
                                println!("--------------------------------------------------------------------------");
                                println!("target [target address]   specify the target which should be stress tested");
                                println!("start                     start stress test");
                                println!("stop                      stop stress test");
                                println!("log                       get current results");
                                println!("help                      get all possible commands");
                                println!("--------------------------------------------------------------------------");
                            }
                            _ => paddr.send(answer.to_owned(), None),
                        }
                    }
                    Err(e) => println!("error: {}", e),
                }
            }
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        println!("init InputActor");
    }
}

struct CommunicationActor {
    id: i32,
    context: Option<Context<Message>>,
    univserve: Arc<mpi::environment::Universe>,
}
impl CommunicationActor {
    fn new() -> CommunicationActor {
        CommunicationActor {
            context: None,
            id: 0,
            univserve: Arc::new(mpi::initialize_with_threading(Threading::Multiple).expect("unwrap 3").0),
        }
    }
}
impl Actor<Message> for CommunicationActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();
        println!("commactor msg : {}", message);
        if vec[0] == "broadcast" {
            let msg_vec: Vec<i32> = vec[1].as_bytes().into_iter().map(|i| *i as i32).collect();
            for x in 0..self.univserve.world().size() {
                self.univserve.world().process_at_rank(x)
                    .ready_send(&msg_vec[..]);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        self.id = self.univserve.world().rank();


        let processor = mpi::environment::processor_name().expect("unwrap 4");
        println!(
            "CommunicationActor from Host : {}, process {} of {}! Starting..",
            processor,
            self.univserve.world().rank() + 1,
            self.univserve.world().size()
        );

        let ctxMaster: &Context<Message> = self.context.as_ref().expect("");
        let master = MasterActor::new();
        ctxMaster.register_actor(master);
        let addressMaster = ctxMaster.own_address.clone();

        //Input Actor
        if self.univserve.world().rank() == 0 {
            let ctx: &Context<Message> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(InputActor { context: None });
            child_addr.send("watch input".to_owned(), None);
        }

        // TODO
        // DEN ACTOR IN DEN THREAD!!!
        let univeryCopy = self.univserve.clone();

        std::thread::spawn(move || {
            println!("hello Thread started");
            
            mpi::request::scope(|scope| {
            
                loop {

                    let (msg, status) = univeryCopy.world()
                        .any_process()
                        .receive_vec::<Rank>();

                    let msgString: String = msg
                        .into_iter()
                        .map(|i| (i as u8) as char)
                        .collect::<String>();
                    // addressMaster.send(
                    //     format!("input {}", msgString)
                    // );

                    println!("msg at {} : {}", mpi::environment::processor_name().unwrap(), msgString);
                }
            });
        });
    }
}

pub fn run() {
    println!("init");
    ActorSystem::start(|| {
        let spawning_addr = ActorSystem::register_actor(CommunicationActor::new(), None);
    });
}
