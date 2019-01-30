use crate::context::*;
use crate::address::*;

pub trait Actor {
    fn handle(&mut self, message: String, origin_address: Option<Address>);
    fn receive_context(&mut self, context: Context);
}