use std::sync::mpsc::*;

use crate::message::*;

#[derive(Clone)]
pub struct Address<M> {
    pub sender: Sender<Envelope<M>>,
}

// TODO: handle panic that send() can cause
// TODO: Is there a way to handle a future returned from this method and therefore be able to
// remove the wait()?
// TODO: Can we get around this clone()?
// TODO: wait() returns a result -> can contain an error. How should that error be handled?
impl<M> Address<M> {
    pub fn send(&self, message: M, origin_address: Option<Address<M>>) {
         let result = self.sender
            .clone()
            .send(Envelope { message: message, origin_address: origin_address });
        match result {
            Err(e) => println!("error sending message: {}", e),
            _ => (),
        }
    }
}