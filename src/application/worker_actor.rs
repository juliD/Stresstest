use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::message::Message;

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
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        match message {
            Message::Start => {
                if self.status {
                    self.status = false;
                    return;
                }
                let ctx = self.context.as_ref().expect("unwrapping context");
                let paddr = ctx
                    .parent_address
                    .as_ref()
                    .expect("unwrapping parent address");

                self.request().map_err(|e| println!("{}", e));
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
