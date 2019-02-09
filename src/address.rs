use std::sync::mpsc::*;

use crate::message::*;

/// Represents the only way to communicate with the associated actor by sending messages.
/// The type parameter represents the message type allowed in this `ActorSystem`.
#[derive(Clone)]
pub struct Address<M>
where
    M: Clone,
{
    pub sender: Sender<Envelope<M>>,
}

impl<M> Address<M>
where
    M: Clone,
{
    /// Sends a message to the mailbox of the actor with this address.
    pub fn send(&self, message: M, origin_address: Option<Address<M>>) {
        let result = self.sender.send(Envelope {
            message: message,
            origin_address: origin_address,
        });
        match result {
            Err(e) => println!("encountered an error sending a message: {}", e),
            _ => (),
        }
    }

    /// Shorthand for sending a message anonymously.
    pub fn senda(&self, message: M) {
        self.send(message, None);
    }

    pub fn send_self(&self, message: M) {
        self.send(message, Some((*self).clone()));
    }
}
