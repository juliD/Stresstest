extern crate futures;
extern crate tokio;

use futures::sync::mpsc::*;
use tokio::prelude::*;

use crate::message::*;

#[derive(Clone)]
pub struct Address {
    pub sender: Sender<Envelope>,
}

// TODO: handle panic that send() can cause
// TODO: Is there a way to handle a future returned from this method and therefore be able to
// remove the wait()?
// TODO: Can we get around this clone()?
// TODO: wait() returns a result -> can contain an error. How should that error be handled?
impl Address {
    pub fn send(&self, message: Box<Message>, origin_address: Option<Address>) {
         let result = self.sender
            .clone()
            .send(Envelope { message: message, origin_address: origin_address.clone() })
            .wait();
        match result {
            Err(e) => println!("error sending message: {}", e),
            _ => (),
        }
    }
}