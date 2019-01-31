extern crate futures;
extern crate tokio;

use crate::actor_system::*;
use crate::actor::*;
use crate::address::*;

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
         address.send(message, Some(self.own_address.clone()));
    }
}
