extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use std::hash::{Hash, Hasher};
use tokio::prelude::*;

use crate::message::*;

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
}