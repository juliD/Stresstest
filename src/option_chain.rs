use crate::message::*;

pub trait OptionChainExt {
    fn apply<F>(self, f: F) -> Option<Box<Message>>
    where
        F: FnMut(Box<Message>) -> Option<Box<Message>>;
}

impl OptionChainExt for Option<Box<Message>> {
    fn apply<F>(self, mut f: F) -> Option<Box<Message>>
    where
        F: FnMut(Box<Message>) -> Option<Box<Message>>,
    {
        if self.is_some() {
            return f(self.unwrap());
        }
        None
    }
}