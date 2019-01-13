extern crate futures;

use futures::sync::mpsc::*;
use tokio::prelude::*;

use crate::message::Envelope;

pub struct Dispatcher {}
impl Dispatcher {
    pub fn run_blocking<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        tokio::run(future::ok(()).map(move |_| {
            f();
        }));
    }

    pub fn run_background<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        tokio::spawn(future::ok(()).map(move |_| {
            f();
        }));
    }

    pub fn handle_stream_blocking<F>(receiver: Receiver<Envelope>, f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        tokio::run(receiver.map(f).collect().then(|_| Ok(())));
    }

    pub fn handle_stream_background<F>(receiver: Receiver<Envelope>, f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        tokio::spawn(receiver.map(f).collect().then(|_| Ok(())));
    }
}