extern crate futures;

use futures::sync::mpsc::*;
use std::sync::mpsc;

use crate::actor::*;
use crate::address::*;
use crate::context::*;
use crate::message::Envelope;
// use crate::tokio_util::TokioUtil;
use crate::thread_utils::ThreadUtils;

const CHANNEL_BUFFER_SIZE: usize = 1_024;
const SYSTEM_STARTING_MESSAGE: &str = "setting up system";
const SYSTEM_STARTUP_FINISHED_MESSAGE: &str = "setup done";

pub struct ActorSystem {}

impl ActorSystem {
    pub fn start<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        println!("{}", SYSTEM_STARTING_MESSAGE);
        f();
        println!("{}", SYSTEM_STARTUP_FINISHED_MESSAGE);
        loop {
            // TODO: better solution to keep main thread alive?
        }
    }

    pub fn register_actor<A>(mut actor: A, parent_address: Option<Address>) -> Address
    where
        A: Actor + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel::<Envelope>();

        let child_address = Address { sender: sender };
        let parent_address_clone = parent_address.map(|address: Address| address.clone());

        let context = Context {
            parent_address: parent_address_clone,
            own_address: child_address.clone(),
        };
        actor.receive_context(context);
        println!("before handle_stream_background");
        ThreadUtils::handle_stream_background(receiver, move |envelope| {
            actor.handle(envelope.message);
        });
        println!("after handle_stream_background");
        child_address
    }
}
