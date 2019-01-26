extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use tokio::prelude::*;

use crate::message::*;
use crate::context::*;

#[derive(Clone)]
pub struct Address {
    pub sender: Sender<Envelope>,
}

// TODO: handle panic that send() can cause
// TODO: Is there a way to handle a future returned from this method and therefore be able to
// remove the wait()?
impl Address {
    pub fn send(&self, message: String) {
         self.sender
            .clone()
            .send(Envelope { message: message })
            .wait();
    }
}

pub trait Actor {
    fn handle(&mut self, message: String);
    fn receive_context(&mut self, context: Context);
}