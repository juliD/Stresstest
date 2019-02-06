use crate::address::*;
use crate::context::*;

/// The basic building block of an `ActorSystem`-based application.
/// It represents an independent part of computation or logic with private state.
/// It does not share any state with other parts of the application.
/// `Actor`s communicate exclusively by sending messages over provided channels,
/// accessible through an associated `Address`.
///
/// The type parameter represents the type of messages that will be passed among `Actor`s in this `ActorSystem`.
/// It is strongly recommended to make use of an `enum` as message type to achieve higher flexibility.
pub trait Actor<M>
where
    M: Clone,
{
    /// This function will be called exactly once, when the `Actor`is created.
    /// It is guaranteed to be called before the first execution of `handle`.
    fn start(&mut self, context: Context<M>);

    /// This function will be called each time the `Actor` receives a message.
    fn handle(&mut self, message: M, origin_address: Option<Address<M>>);
}
