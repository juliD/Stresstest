extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use tokio::prelude::*;

use crate::message::*;
use crate::context::*;

pub type ActorId = u64;

#[derive(Clone)]
pub struct Address {
    pub sender: Sender<Envelope>,
}

impl Address {
    pub fn send(&self, message: String) {
        self.sender
            .clone()
            .send_all(stream::once(Ok(Envelope { message: message })))
            .wait()
            .ok();
    }
}

pub trait Actor {
    fn handle(&mut self, message: String);
    fn receive_context(&mut self, context: Context);
}