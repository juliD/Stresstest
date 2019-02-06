use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::env;
use std::io::{self, BufRead};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::message::*;

use crate::application::address_parsing::*;
use crate::application::tcp_listen_actor::TcpListenActor;
use crate::application::input_actor::InputActor;
use crate::application::message::Message;
use crate::application::utils::*;


pub struct WorkerActor {
    pub id: u32,
    pub context: Option<Context<Message>>,
    pub target: String,
    pub status: bool,
}
impl WorkerActor {
    fn request(&self) -> Result<(), Box<std::error::Error>> {
        let res = reqwest::get(&self.target)?;
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
            _ => println!("WorkerActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}