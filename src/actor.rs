use crate::context::*;
use crate::address::*;
use crate::message::*;

pub trait Actor {
    fn handle(&mut self, message: Box<Message>, origin_address: Option<Address>);
    fn receive_context(&mut self, context: Context);
}