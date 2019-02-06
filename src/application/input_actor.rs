use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use std::io::{self, BufRead};

use crate::application::message::Message;
use crate::application::message_serialization::*;

pub struct InputActor {
    pub context: Option<Context<Message>>,
}
impl Actor<Message> for InputActor {
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        if let Message::StartWatchInput(master_addr) = message {
            println!("InputActor starts listening");

            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(input) => {
                        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
                        let message_option = parse_message(input.as_ref());

                        match message_option {
                            Some(msg) => ctx.send(&master_addr, msg),
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
