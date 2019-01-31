use crate::address::*;

#[derive(Clone)]
pub struct Envelope {
    pub message: String,
    pub origin_address: Option<Address>,
}