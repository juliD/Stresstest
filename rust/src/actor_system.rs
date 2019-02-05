use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::actor::*;
use crate::address::*;
use crate::context::*;
use crate::message::Envelope;
use crate::thread_utils::ThreadUtils;

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
            thread::sleep(Duration::from_millis(1000));
        }
    }

    pub fn register_actor<M, A>(mut actor: A, parent_address: Option<Address<M>>) -> Address<M>
    where
        M: Clone + Send + 'static,
        A: Actor<M> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel::<Envelope<M>>();

        let child_address = Address { sender: sender };
        let parent_address_clone = parent_address.map(|address: Address<M>| address.clone());

        let context = Context {
            parent_address: parent_address_clone,
            own_address: child_address.clone(),
        };
        actor.start(context);
        ThreadUtils::handle_stream_background(receiver, move |envelope: Envelope<M>| {
            actor.handle(envelope.message, envelope.origin_address);
        });
        child_address
    }
}
