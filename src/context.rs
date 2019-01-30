extern crate futures;
extern crate tokio;

use tokio::prelude::*;
use crate::actor_system::*;
use crate::actor::*;
use crate::address::*;
use crate::message::*;

pub struct Context {
    pub parent_address: Option<Address>,
    pub own_address: Address,
}
impl Context {

    pub fn register_actor<A>(&self, actor: A) -> Address
    where
        A: Actor + Send + 'static,
    {
        ActorSystem::register_actor(actor, Some(self.own_address.clone()))
    }

    pub fn send(&self, address: &Address, message: String) {
         let result = address.sender
            .clone()
            .send(Envelope { message: message, sender: Some(self.own_address.clone()) })
            .wait();
        match result {
            Err(e) => println!("error sending message: {}", e),
            _ => (),
        }
    }
}
