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
// TODO: Can we get around this clone()?
impl Address {
    pub fn send(&self, message: String) {
         self.sender
            .clone()
            .send(Envelope { message: message })
            .wait();
    }
}