use std::collections::LinkedList;
use std::net::IpAddr;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::address_parsing::*;
use crate::application::message::Message;
use crate::application::worker_actor::WorkerActor;
use crate::application::config_actor::AppConfig;

pub struct MasterActor {
    pub workers: LinkedList<Address<Message>>,
    pub tcp_actor_addr: Address<Message>,
    pub input_actor_addr: Option<Address<Message>>,
    pub config_actor_addr: Address<Message>,
    pub child_id_counter: u32,
    pub context: Option<Context<Message>>,
    pub master: bool,
    pub counter: u64,
}
impl MasterActor {
    pub fn new(
        is_master: bool,
        tcp_actor_addr: Address<Message>,
        input_actor_addr: Option<Address<Message>>,
        config_actor_addr: Address<Message>,
    ) -> MasterActor {
        MasterActor {
            workers: LinkedList::new(),
            child_id_counter: 0,
            context: None,
            master: is_master,
            counter: 0,
            tcp_actor_addr: tcp_actor_addr,
            input_actor_addr: input_actor_addr,
            config_actor_addr: config_actor_addr,
        }
    }

    fn send_tcp_message(&self,message: Message) {
        self.tcp_actor_addr
            .send(Message::SendTcpMessage(Box::new(message)), None);
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
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");

        match message {
            Message::ReportRequests(count) => {                
                if self.master {
                    self.counter += count;
                } else {
                    self.counter += count;
                    if self.counter>=50{                                              
                        self.send_tcp_message(Message::ReportRequests(self.counter));
                        self.counter = 0; 
                    }
                    
                }
            }
            Message::Start => {
                if self.master {
                    self.counter = 0;
                    self.send_tcp_message(Message::Start);
                } else {
                    println!("MasterActor starting");
                    broadcast_children(ctx, &self.workers, self.child_id_counter, Message::Start);
                }
            }
            Message::Stop => {
                if self.master {
                    self.send_tcp_message(Message::Stop);
                } else {
                    println!("MasterActor stopped");
                    broadcast_children(ctx, &self.workers, self.child_id_counter, Message::Stop);
                    self.send_tcp_message(Message::ReportRequests(self.counter));
                    self.counter = 0;
                }
            }            
            Message::Log => {
                println!("Requests sent: {}", self.counter);
            }
            Message::SetTarget(target_address_raw) => {
                println!("Address: {}",target_address_raw);
                if !self.master {
                    // TODO: Use String in target method
                    let target_address_is_valid = verify_target_address(&target_address_raw);
                    if target_address_is_valid {
                        println!("valid target {}", target_address_raw);
                        broadcast_children(
                            ctx,
                            &self.workers,
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
        println!("init MasterActor");
        self.context = Some(context);


        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
        //get Config
        ctx.send(
            &self.config_actor_addr,
            Message::StartWatchingConfig(ctx.own_address.clone()),
        );
        
        if self.master {
            match self.input_actor_addr.as_ref() {
                Some(addr) => ctx.send(&addr, Message::StartWatchInput(ctx.own_address.clone())),
                None => println!("MasterActor has no InputActor"),
            }
            
            // start listening for worker connections
            ctx.send(
                &self.tcp_actor_addr,
                Message::StartListenForTcp(ctx.own_address.clone()),
            );

        } else {
            //create as many actors as cores available
            let num = num_cpus::get();
            for _ in 0..num {
                self.child_id_counter += 1;
                let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
                let child_addr = ctx.register_actor(WorkerActor {
                    id: self.child_id_counter,
                    context: None,
                    target: "http://httpbin.org/ip".to_owned(),
                    status: false,
                });
                self.workers.push_back(child_addr.clone());
            }
            // connect to master
            ctx.send(&self.tcp_actor_addr, Message::ConnectToMaster(ctx.own_address.clone()));
        }
    }
}
