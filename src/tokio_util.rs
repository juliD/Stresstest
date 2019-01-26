extern crate futures;

use futures::sync::mpsc::*;
use tokio::prelude::*;

use crate::message::Envelope;

// TODO: Less code duplication by returning futures from a util function?

pub struct TokioUtil {}
impl TokioUtil {
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

    // TODO: remove as this function is never called
    /* pub fn handle_stream_blocking<F>(receiver: Receiver<Envelope>, mut f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        // TODO: Is the new solution better than the old one? Not sure, had to add the "mut" above ...
        // tokio::run(receiver.map(f).collect().then(|_| Ok(())));
        tokio::run(receiver.for_each(move |msg: Envelope| {
            Ok(f(msg))
        }));
    } */

    pub fn handle_stream_background<F>(receiver: Receiver<Envelope>, mut f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        // TODO: Is the new solution better than the old one? Not sure, had to add the "mut" above ...
        tokio::spawn(receiver.for_each(move |msg: Envelope| {
            f(msg);
            Ok(())
        }));
        // tokio::spawn(receiver.map(f).collect().then(|_| Ok(())));
    }
}