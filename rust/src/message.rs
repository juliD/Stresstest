use crate::address::*;

#[derive(Clone)]
pub struct Envelope<M> {
    pub message: M,
    pub origin_address: Option<Address<M>>,
}