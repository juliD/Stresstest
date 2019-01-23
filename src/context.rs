use crate::router::*;
use crate::actor::*;

pub struct Context {
    pub parent_address: Option<Address>,
    pub own_address: Address,
}
impl Context {

    pub fn register_actor<A>(&self, actor: A) -> Address
    where
        A: Actor + Send + 'static,
    {
        Router::register_actor(actor, Some(self.own_address.clone()))
    }
}
