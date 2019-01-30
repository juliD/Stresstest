use std::sync::mpsc;
use std::thread;

use crate::message::Envelope;

pub struct ThreadUtils {}
impl ThreadUtils {
    pub fn run_blocking<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        f();
    }

    pub fn run_background<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        thread::spawn(move || {
            f();
        });
    }

    pub fn handle_stream_background<F>(receiver: mpsc::Receiver<Envelope>, mut f: F)
    where
        F: FnMut(Envelope) + 'static + Send,
    {
        thread::spawn(move || {
            for message in receiver {
                f(message);
            }
        });
    }
}
