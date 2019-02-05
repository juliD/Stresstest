use crate::context::*;
use crate::address::*;

pub trait Actor<M> {
    fn handle(&mut self, message: M, origin_address: Option<Address<M>>);
    fn start(&mut self, context: Context<M>);
}