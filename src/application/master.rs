extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::message::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::LinkedList;
use std::env;
use std::io::{self, BufRead};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

#[derive(Clone)]
enum Message {
    // MasterActor
    Log,
    ReportRequests(u64),
    Help,
    IncomingTcpMessage(String),
    // MasterActor + WorkerActor
    Start,
    Stop,
    SetTarget(String),
    // TcpActor
    StartListenTcp,
    // InputActor
    StartWatchInput,
}
// type Message = String;

struct WorkerActor {
    id: u32,
    context: Option<Context<Message>>,
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

impl Actor<Message> for WorkerActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        match message {
            Message::Start => {
                if self.status {
                    self.status = false;
                    return;
                }
                let ctx = self.context.as_ref().expect("");
                let paddr = ctx.parent_address.as_ref().expect("");
                let own_addr = ctx.own_address.clone();

                self.request();
                ctx.send(&paddr, Message::ReportRequests(1));
                ctx.send(&ctx.own_address, Message::Start);
            }
            Message::SetTarget(target) => {
                self.target = target;
            }
            Message::Stop => {
                println!("WorkerActor stopped");
                self.status = true;
            }
            _ => println!("WorkerActor received unknown message")
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
                stream
                    .write(serialize_string_message(Message::ReportRequests(1)).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    }
}

fn broadcast_children(
    ctx: &Context<Message>,
    list: &LinkedList<Address<Message>>,
    count: u32,
    message: Message,
) {
    let mut iter = list.iter();
    iter.next();

    for _ in 0..count - 1 {
        match iter.next() {
            Some(c) => {
                ctx.send(&c, message.clone());
            }
            _ => {}
        }
    }
}

impl Actor<Message> for MasterActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("");

        match message {
            Message::ReportRequests(count) => {
                if self.master {
                    self.counter += count;
                } else {
                    self.send_master_count();
                }
            }
            Message::Start => {
                if self.master {
                    self.counter = 0;
                    self.send_slaves(serialize_string_message(Message::Start));
                } else {
                    println!("MasterActor starting");
                    broadcast_children(ctx, &self.children, self.child_id_counter, Message::Start);
                }
            }
            Message::Stop => {
                if self.master {
                    self.counter = 0;
                    self.send_slaves(serialize_string_message(Message::Stop));
                } else {
                    println!("MasterActor stopped");
                    broadcast_children(ctx, &self.children, self.child_id_counter, Message::Stop);
                }
            }
            Message::Log => {
                println!("Requests sent: {}", self.counter);
            }
            Message::SetTarget(target) => {
                if !self.master {
                    //TODO:check if target correct
                    broadcast_children(
                        ctx,
                        &self.children,
                        self.child_id_counter,
                        Message::SetTarget(target),
                    );
                }
            }
            Message::Help => {
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
            _ => println!("MasterActor received unknown message")
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        println!("init MasterActor");

        let mut port = 3333;
        if self.master {
            //Input Actor
            self.child_id_counter += 1;
            let ctx: &Context<Message> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(InputActor {
                id: self.child_id_counter,
                context: None,
            });
            self.children.push_back(child_addr.clone());
            ctx.send(&child_addr, Message::StartWatchInput);
            port = 3001;
        }

        //create TCPListener
        self.child_id_counter += 1;
        let ctx: &Context<Message> = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(TcpListenActor {
            context: None,
            port: port,
        });
        self.children.push_back(child_addr.clone());
        ctx.send(&child_addr, Message::StartListenTcp);

        if !self.master {
            //create as many actors as cores available
            let num = num_cpus::get();
            for x in 0..num {
                self.child_id_counter += 1;
                let ctx: &Context<Message> = self.context.as_ref().expect("");
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
}

struct TcpListenActor {
    context: Option<Context<Message>>,
    port: u32,
}
impl Actor<Message> for TcpListenActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
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
                            match parse_string_message(message) {
                                Some(actor_message) => paddr.send(actor_message, Some(paddr.clone())),
                                None => println!("could not parse tcp message: {}", message)
                            };
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
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        println!("init InputActor");
    }
}

fn parse_string_message(message: &str) -> Option<Message> {
    match message {
        "start" => Some(Message::Start),
        "stop" => Some(Message::Stop),
        "log" => Some(Message::Log),
        "help" => Some(Message::Help),
        _ => {
            let split = message.split(" ");
            let vec: Vec<&str> = split.collect();
            match vec[0] {
                "reportrequests" => Some(Message::ReportRequests(vec[1].parse().unwrap())),
                _ => None,
            }
        }
    }
}

fn serialize_string_message(message: Message) -> String {
    match message {
        Message::Start => "start".to_owned(),
        Message::Stop => "stop".to_owned(),
        Message::Log => "log".to_owned(),
        Message::Help => "help".to_owned(),
        Message::ReportRequests(count) => format!("reportrequests {}", count),
        _ => panic!("failed to serialize actor message"),
    }
}

struct InputActor {
    id: u32,
    context: Option<Context<Message>>,
}
impl Actor<Message> for InputActor {
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        if let Message::StartWatchInput = message {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(input) => {
                        let ctx: &Context<Message> = self.context.as_ref().expect("");
                        let paddr: &Address<Message> = ctx.parent_address.as_ref().expect("");
                        let message_option = parse_string_message(input.as_ref());

                        match message_option {
                            Some(msg) => ctx.send(&paddr, msg),
                            None => {
                                println!("Not a valid input. Write help to get a list of commands.")
                            }
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

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");

    println!("init");
    ActorSystem::start(move || {
        ActorSystem::register_actor(MasterActor::new(is_master_bool), None);
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
