use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::message::Message;


pub struct LogActor {
    pub id: u32,
    pub context: Option<Context<Message>>,
}

impl Actor<Message> for LogActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {}

    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}