extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::hash::{Hash, Hasher};
use tokio::prelude::*;

use crate::message::*;
use crate::router::*;

pub type ActorId = u64;

#[derive(Clone)]
pub struct Address {
    pub sender: Sender<Envelope>,
}

impl Address {
    pub fn send(&self, message: Message) {
        self.sender
            .clone()
            .send_all(stream::once(Ok(Envelope { message: message })))
            .wait()
            .ok();
    }
}

pub trait Actor {
    fn handle(&mut self, message: Message);
    fn receive_context(&mut self, context: Context);
}

pub struct Context {
    pub parent_address: Option<Address>,
    pub own_address: Address,
}
impl Context {

    pub fn register_actor<A>(&self, actor: A) -> Address
    where
        A: Actor + Send + 'static,
    {
        Router::register_actor(actor, Some(self.own_address.clone()))
    }
}
