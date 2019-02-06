use std::sync::mpsc;
use std::thread;

use crate::message::Envelope;

/// Provides functions to manage code execution in other threads and thus encapsulates runtime dependencies.
pub struct ThreadUtils {}
impl ThreadUtils {

    /// Run the given closure in a new thread.
    pub fn run_background<F>(f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        thread::spawn(move || {
            f();
        });
    }

    /// Maps each incoming message of the given `Receiver` by applying the given closure to it.
    /// This is done in a separate thread.
    pub fn handle_stream_background<M, F>(receiver: mpsc::Receiver<Envelope<M>>, mut f: F)
    where
        M: Clone + Send + 'static,
        F: FnMut(Envelope<M>) + 'static + Send,
    {
        thread::spawn(move || {
            for message in receiver {
                f(message);
            }
        });
    }
}
