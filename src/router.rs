extern crate futures;

use futures::sync::mpsc::*;
use std::thread;
use std::time::Duration;

use crate::actor::*;
use crate::dispatcher::Dispatcher;
use crate::message::Envelope;
use crate::context::*;

pub struct Router {}

impl Router {
    pub fn start<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        Dispatcher::run_blocking(move || {
            println!("setting up system");
            f();
            println!("setup done");
            loop {
                thread::sleep(Duration::from_millis(100))
            }
        });

        // run blocking stream to keep system alive
        // println!("run blocking stream to keep system alive");
        // let (_, system_receiver) = channel::<Envelope>(8);
        // Dispatcher::handle_stream_blocking(system_receiver, move |_| {});
    }

    pub fn register_actor<A>(actor: A, parent_address: Option<Address>) -> Address
    where
        A: Actor + Send + 'static,
    {
        let (sender, receiver) = channel::<Envelope>(16);

        let child_address = Address { sender: sender };

        let mut actor_ref = actor;

        let parent_address_clone = match parent_address {
            Some(addr) => Some(addr.clone()),
            None => None,
        };

        let context = Context {
            parent_address: parent_address_clone,
            own_address: child_address.clone(),
        };
        actor_ref.receive_context(context);

        Dispatcher::handle_stream_background(receiver, move |envelope| {
            actor_ref.handle(envelope.message);
        });
        child_address
    }
}
