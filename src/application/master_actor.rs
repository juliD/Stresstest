use std::io::Write;
use std::net::TcpStream;
use std::collections::LinkedList;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::address_parsing::*;
use crate::application::worker_actor::WorkerActor;
use crate::application::tcp_listen_actor::TcpListenActor;
use crate::application::input_actor::InputActor;
use crate::application::message::Message;
use crate::application::utils::*;


pub struct MasterActor {
    pub children: LinkedList<Address<Message>>,
    pub child_id_counter: u32,
    pub context: Option<Context<Message>>,
    pub master: bool,
    pub counter: u64,
}
impl MasterActor {
    pub fn new(is_master: bool) -> MasterActor {
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
            Message::SetTarget(target_address_raw) => {
                if !self.master {
                    // TODO: Use String in target method
                    let target_address_is_valid =
                        verify_target_address(&target_address_raw);
                    if target_address_is_valid {
                        println!("valid target {}", target_address_raw);
                        broadcast_children(
                            ctx,
                            &self.children,
                            self.child_id_counter,
                            Message::SetTarget(target_address_raw),
                        );
                    } else {
                        println!("invalid target address (please enter <IP>:<port>)");
                    }
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
            _ => println!("MasterActor received unknown message"),
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