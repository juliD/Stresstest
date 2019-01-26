extern crate futures;

use futures::sync::mpsc::*;

use crate::actor::*;
use crate::tokio_util::TokioUtil;
use crate::message::Envelope;
use crate::context::*;
use crate::address::*;

const CHANNEL_BUFFER_SIZE: usize = 1_024;
const SYSTEM_STARTING_MESSAGE: &str = "setting up system";
const SYSTEM_STARTUP_FINISHED_MESSAGE: &str = "setup done";

pub struct Router {}

impl Router {
    pub fn start<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        TokioUtil::run_blocking(move || {
            println!("{}", SYSTEM_STARTING_MESSAGE);
            f();
            println!("{}", SYSTEM_STARTUP_FINISHED_MESSAGE);
        });

        // TODO: remove?
        // run blocking stream to keep system alive
        // println!("run blocking stream to keep system alive");
        // let (_, system_receiver) = channel::<Envelope>(8);
        // TokioUtil::handle_stream_blocking(system_receiver, move |_| {});
    }

    pub fn register_actor<A>(mut actor: A, parent_address: Option<Address>) -> Address
    where
        A: Actor + Send + 'static,
    {
        let (sender, receiver) = channel::<Envelope>(CHANNEL_BUFFER_SIZE);

        let child_address = Address { sender: sender };
        let parent_address_clone = parent_address.map(|address: Address| address.clone());

        let context = Context {
            parent_address: parent_address_clone,
            own_address: child_address.clone(),
        };
        actor.receive_context(context);

        TokioUtil::handle_stream_background(receiver, move |envelope| {
            actor.handle(envelope.message);
        });
        child_address
    }
}
