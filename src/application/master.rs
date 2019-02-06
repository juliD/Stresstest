extern crate actor_model;
extern crate byteorder;
extern crate reqwest;
use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use std::collections::LinkedList;

use std::env;
use std::io::{self, BufRead};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

use crate::application::address_parsing;
use crate::application::input_processing;

struct WorkerActor {
    id: u32,
    context: Option<Context<String>>,
    target: String,
    status: bool,
}
impl WorkerActor {
    fn request(&self) -> Result<(), Box<std::error::Error>> {
        let mut res = reqwest::get(&self.target)?;
        //println!("Status = {}", res.status());

        Ok(())
    }
}

impl Actor<String> for WorkerActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();

        match vec[0] {
            "start" => {
                if self.status {
                    self.status = false;
                    return;
                }
                let ctx = self.context.as_ref().expect("");
                let paddr = ctx.parent_address.as_ref().expect("");
                let own_addr = ctx.own_address.clone();

                self.request();
                ctx.send(&paddr, "1".to_string());
                ctx.send(&own_addr, "start".to_string());
            }
            "target" => {
                self.target = vec[1].parse().unwrap();
            }
            "stop" => {
                println!("Stopped");
                self.status = true;
            }
            _ => {}
        }
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
    }
}

struct LogActor {
    id: u32,
    context: Option<Context<String>>,
}

impl Actor<String> for LogActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {}

    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
    }
}

struct MasterActor {
    children: LinkedList<Address<String>>,
    child_id_counter: u32,
    context: Option<Context<String>>,
    master: bool,
    counter: u64,
}
impl MasterActor {
    fn new(is_master: bool) -> MasterActor {
        MasterActor {
            children: LinkedList::new(),
            child_id_counter: 0,
            context: None,
            master: is_master,
            counter: 0,
        }
    }
    fn send_slaves(&self, message: String) {
        //TODO: send to all known slaves
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                stream.write(message.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
    fn send_master_count(&self) {
        match TcpStream::connect("localhost:3001") {
            Ok(mut stream) => {
                stream.write("1".as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}

fn broadcast_children(
    ctx: &Context<String>,
    list: &LinkedList<Address<String>>,
    count: u32,
    message: String,
) {
    let mut iter = list.iter();
    iter.next();

    for _ in 0..count - 1 {
        match iter.next() {
            Some(c) => {
                ctx.send(&c, message.to_owned());
            }
            _ => {}
        }
    }
}

impl Actor<String> for MasterActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {
        let ctx: &Context<String> = self.context.as_ref().expect("");
        let (input1, input2) = input_processing::parse_user_input(&message);
        match input1 {
            Some("1") => {
                if self.master {
                    self.counter += 1;
                } else {
                    self.send_master_count();
                    println!("Sent count");
                }
            }
            Some("start") => {
                if self.master {
                    self.counter = 0;
                    self.send_slaves(message);
                } else {
                    println!("Started");
                }
                broadcast_children(
                    ctx,
                    &self.children,
                    self.child_id_counter,
                    "start".to_string(),
                );
            }
            Some("stop") => {
                if self.master {
                    self.counter = 0;
                    self.send_slaves(message);
                } else {
                    println!("Stopped");
                }
                broadcast_children(
                    ctx,
                    &self.children,
                    self.child_id_counter,
                    "stop".to_string(),
                );
            }
            Some("log") => {
                println!("Requests sent: {}", self.counter);
            }
            Some("target") => {
                match input2 {
                    Some(target_address_raw) => {
                        let target_address_is_valid = address_parsing::verify_target_address(target_address_raw);
                        if target_address_is_valid {
                            println!("valid target {}", target_address_raw);
                            broadcast_children(ctx, &self.children, self.child_id_counter, message);
                        } else {
                            println!("invalid target address (please enter <IP>:<port>)");
                        }
                    },
                    _ => {
                        println!("please enter a target adress after 'target'");
                    }
                }
            }
            Some("help") => {
                println!(
                    "--------------------------------------------------------------------------"
                );
                println!(
                    "target [target address]   specify the target which should be stress tested"
                );
                println!("start                     start stress test");
                println!("stop                      stop stress test");
                println!("log                       get current results");
                println!("help                      get all possible commands");
                println!(
                    "--------------------------------------------------------------------------"
                );
            }
            _ => {
                println!("This is what I got: -{}-", message);

                println!("Not a valid input. Write help to get a list of commands.")
            }
        }
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init MasterActor");

        let mut port = 3333;
        if self.master {
            //Input Actor
            self.child_id_counter += 1;
            let ctx: &Context<String> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(InputActor {
                id: self.child_id_counter,
                context: None,
            });
            self.children.push_back(child_addr.clone());
            ctx.send(&child_addr, "watch input".to_owned());
            port = 3001;
        }

        //create TCPListener
        self.child_id_counter += 1;
        let ctx: &Context<String> = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(TcpListenActor {
            context: None,
            port: port,
        });
        self.children.push_back(child_addr.clone());
        ctx.send(&child_addr, "listen".to_string());

        //create as many actors as cores available
        let num = num_cpus::get();
        for x in 0..num {
            self.child_id_counter += 1;
            let ctx: &Context<String> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(WorkerActor {
                id: self.child_id_counter,
                context: None,
                target: "http://httpbin.org/ip".to_owned(),
                status: false,
            });
            self.children.push_back(child_addr.clone());
        }
    }
}

struct TcpListenActor {
    context: Option<Context<String>>,
    port: u32,
}
// TODO: This actor will block after it receives the first message. Is that wanted behavior?
impl Actor<String> for TcpListenActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {
        let addr = format!("localhost:{}", self.port);
        let listener = TcpListener::bind(addr).unwrap();

        // accept connections and process them, spawning a new thread for each one
        println!("master listening on port {}", self.port);
        for stream in listener.incoming() {
            let ctx = self.context.as_ref().expect("");
            let paddr = ctx.parent_address.as_ref().expect("").clone();
            match stream {
                Ok(stream) => {
                    //println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || {
                        handle_messages(stream, |message: &str| {
                            //println!("received message {}", message);
                            paddr.send(message.to_string(), Some(paddr.clone()));
                        });
                    });
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init TcpListenActor");
    }
}

struct InputActor {
    id: u32,
    context: Option<Context<String>>,
}
// TODO: This actor will block after it receives the "watch input" message. Is that wanted behavior?
impl Actor<String> for InputActor {
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {
        if message == "watch input" {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(input) => {
                        let ctx: &Context<String> = self.context.as_ref().expect("");
                        let paddr: &Address<String> = ctx.parent_address.as_ref().expect("");

                        ctx.send(&paddr, input.to_owned());
                    }
                    Err(e) => println!("error: {}", e),
                }
            }
        }
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init InputActor");
    }
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");

    println!("init");
    ActorSystem::start(move || {
        let spawning_addr = ActorSystem::register_actor(MasterActor::new(is_master_bool), None);
    });
}

fn handle_messages<T>(mut conn: TcpStream, f: T)
where
    T: FnOnce(&str),
{
    let mut buffer = [0; 512];
    conn.read(&mut buffer).unwrap();
    let message_as_string = String::from_utf8_lossy(&buffer[..]);
    let message_removed_nulls = message_as_string.trim_right_matches(char::from(0));
    // TODO: Wow, this looks weird.
    let message_to_pass_on = &(*message_removed_nulls);
    f(message_to_pass_on);

    let string_response = "Thanks!";
    conn.write(string_response.as_bytes()).unwrap();
    conn.flush().unwrap();
}
