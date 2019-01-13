#[derive(Clone)]
pub struct Message {
    pub payload: String,
}

#[derive(Clone)]
pub struct Envelope {
    pub message: Message,
}