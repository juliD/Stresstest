use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::message::Message;

use std::ops::Drop;

pub struct WorkerActor {
    pub context: Option<Context<Message>>,
    pub target: String,
    pub stopped: bool,
}
impl WorkerActor {
    // TODO: prettify
    fn request(&self) -> Result<(), Box<std::error::Error>> {
        let res = reqwest::get(&self.target)?;

        Ok(())
    }
}

impl Drop for WorkerActor {
    fn drop(&mut self) {
        println!("WorkerActor dropped");
    }
}

impl Actor<Message> for WorkerActor {
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        match message {
            Message::Start => {
                if self.stopped {
                    self.stopped = false;
                    return;
                }
                match self.context {
                    Some(ref ctx) => {
                        let paddr = ctx
                            .parent_address
                            .as_ref()
                            .expect("unwrapping parent address");

                        self.request().map_err(|e| println!("{}", e));
                        ctx.send(&paddr, Message::ReportRequests(1));
                        ctx.send(&ctx.own_address, Message::Start);
                    }
                    None => println!("no context available in WorkerActor")
                }
            }
            Message::SetTarget(target) => {
                self.target = target;
                println!("new target set: {}", self.target);
            }
            Message::Stop => {
                println!("WorkerActor stopped");
                self.stopped = true;
            }
            Message::Kill => {
                self.context = None;
            }
            _ => println!("WorkerActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}
