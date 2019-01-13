extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::hash::{Hash, Hasher};
use tokio::prelude::*;

use crate::message::*;
use crate::router::Context;

pub type ActorId = u64;

#[derive(Clone)]
pub struct ActorAddress {
    pub id: ActorId,
    pub sender: Sender<Envelope>,
}
impl Hash for ActorAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ActorAddress {
    fn eq(&self, other: &ActorAddress) -> bool {
        self.id == other.id
    }
}
impl Eq for ActorAddress {}
impl ActorAddress {
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