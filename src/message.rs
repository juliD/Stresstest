use crate::address::*;

#[derive(Clone)]
pub struct Envelope {
    pub message: String,
    pub sender: Option<Address>,
}