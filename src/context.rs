extern crate futures;
extern crate tokio;

use crate::actor_system::*;
use crate::actor::*;
use crate::address::*;

pub struct Context<M> {
    pub parent_address: Option<Address<M>>,
    pub own_address: Address<M>,
}
impl<M> Context<M> where M: Clone + Send + 'static {

    pub fn register_actor<A>(&self, actor: A) -> Address<M>
    where
        A: Actor<M> + Send + 'static,
    {
        ActorSystem::register_actor(actor, Some(self.own_address.clone()))
    }

    pub fn send(&self, address: &Address<M>, message: M) {
         address.send(message, Some(self.own_address.clone()));
    }
}
