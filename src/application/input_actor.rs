use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use std::io::{self, BufRead};

use crate::application::message::Message;
use crate::application::message_serialization::*;


pub struct InputActor {
    pub id: u32,
    pub context: Option<Context<Message>>,
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
                        let message_option = parse_message(input.as_ref());

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