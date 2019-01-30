use downcast_rs::Downcast;

use crate::address::*;

pub struct Envelope {
    pub message: Box<Message>,
    pub origin_address: Option<Address>,
}


pub trait Message: Downcast + Send {}
impl_downcast!(Message);