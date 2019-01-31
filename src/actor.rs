use crate::context::*;
use crate::address::*;

pub trait Actor<M> {
    fn handle(&mut self, message: M, origin_address: Option<Address<M>>);
    fn receive_context(&mut self, context: Context<M>);
}