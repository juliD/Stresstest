use crate::actor_system::*;
use crate::actor::*;
use crate::address::*;

/// The `Context` is individual to each `Actor` and contains it's own and it's parent's `Address`.
/// Also, it provides convinience functions to register a child actor and send messages.
/// The type parameter represents the message type allowed in this `ActorSystem`.
pub struct Context<M> {
    pub parent_address: Option<Address<M>>,
    pub own_address: Address<M>,
}
impl<M> Context<M> where M: Clone + Send + 'static {

    /// Registers a given `Actor` and sets the `Actor` associated with this `Context` as parent.
    pub fn register_actor<A>(&self, actor: A) -> Address<M>
    where
        A: Actor<M> + Send + 'static,
    {
        ActorSystem::register_actor(actor, Some(self.own_address.clone()))
    }

    /// Sends a message to the `Actor` associated with the given `Address`and sets this `Context`'s `Actor` as origin.
    pub fn send(&self, address: &Address<M>, message: M) {
         address.send(message, Some(self.own_address.clone()));
    }

    /// Sends a message to the `Actor` associated with this `Context`.
    pub fn send_self(&self, message: M) {
         self.own_address.send(message, Some(self.own_address.clone()));
    }
}
