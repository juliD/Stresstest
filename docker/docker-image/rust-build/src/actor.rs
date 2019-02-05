use crate::context::*;

pub trait Actor {
    fn handle(&mut self, message: String);
    fn receive_context(&mut self, context: Context);
}