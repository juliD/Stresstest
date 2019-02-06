use crate::address::*;

/// Wrapper around messages that is sent in `mpsc::channel`s amoung actor threads.
/// The type parameter represents the message type allowed in this `ActorSystem`.
#[derive(Clone)]
pub struct Envelope<M> {
    pub message: M,
    pub origin_address: Option<Address<M>>,
}