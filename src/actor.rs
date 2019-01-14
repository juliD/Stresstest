extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::hash::{Hash, Hasher};
use tokio::prelude::*;

use crate::message::*;
use crate::router::Context;

pub type ActorId = u64;

#[derive(Clone)]
pub struct Address {
    pub id: ActorId,
    pub sender: Sender<Envelope>,
}
impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.id == other.id
    }
}
impl Eq for Address {}
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
    fn handle(&mut self, message: Message, context: Context);
}