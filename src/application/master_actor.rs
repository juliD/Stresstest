use std::collections::LinkedList;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::address_parsing::*;
use crate::application::message::Message;
use crate::application::worker_actor::WorkerActor;

// TODO: put children into vector

pub struct MasterActor {
    pub workers: Vec<Address<Message>>,
    pub tcp_actor_addr: Address<Message>,
    pub input_actor_addr: Option<Address<Message>>,
    pub config_actor_addr: Address<Message>,
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
            workers: Vec::new(),
            context: None,
            master: is_master,
            counter: 0,
            tcp_actor_addr: tcp_actor_addr,
            input_actor_addr: input_actor_addr,
            config_actor_addr: config_actor_addr,
        }
    }

    fn send_tcp_message(&self, message: Message) {
        // TODO: can we remove the box?
        self.tcp_actor_addr
            .send(Message::SendTcpMessage(Box::new(message)), None);
    }

    fn broadcast_children(&self, ctx: &Context<Message>, message: Message) {
        for child in &self.workers {
            ctx.send(&child, message.clone());
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
                    if self.counter >= 50 {
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
                    self.broadcast_children(ctx, Message::Start);
                }
            }
            Message::Stop => {
                if self.master {
                    self.send_tcp_message(Message::Stop);
                } else {
                    println!("MasterActor stopped");
                    self.broadcast_children(ctx, Message::Stop);
                    self.send_tcp_message(Message::ReportRequests(self.counter));
                    self.counter = 0;
                }
            }
            Message::Log => {
                println!("Requests sent: {}", self.counter);
            }
            Message::SetTarget(target_address) => {
                if self.master {
                    // TODO: test whether URL is valid
                    

                    self.send_tcp_message(Message::SetTarget(target_address));
                } else {
                    self.broadcast_children(ctx, Message::SetTarget(target_address));
                }
            }

            Message::Help => {
                println!(
                    "--------------------------------------------------------------------------"
                );
                println!(
                    "target [target address]   specify the target which should be stress tested"
                );
                println!("target                    set target URL of stress test");
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
                let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
                let child_addr = ctx.register_actor(WorkerActor {
                    context: None,
                    target: "http://httpbin.org/ip".to_owned(),
                    stopped: false,
                });
                self.workers.push(child_addr.clone());
            }
            // connect to master
            ctx.send(
                &self.tcp_actor_addr,
                Message::ConnectToMaster(ctx.own_address.clone()),
            );
        }
    }
}
