extern crate futures;

use futures::sync::mpsc::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::actor::*;
use crate::dispatcher::Dispatcher;
use crate::message::Envelope;

pub struct Router {
    actors: HashMap<ActorId, Arc<Mutex<Actor + Send>>>,
    actor_id_counter: ActorId,
}

impl Router {
    pub fn create() -> Router {
        println!("creating ActorSystem");
        Router {
            actors: HashMap::new(),
            actor_id_counter: 0,
        }
    }

    pub fn start<F>(f: F)
    where
        F: FnOnce(Context) + 'static + Send,
    {
        // init actor system
        let actor_system = Arc::new(Mutex::new(Router::create()));

        let context = Context {
            system: actor_system.clone(),
        };
        let context_clone = context.clone();

        Dispatcher::run_blocking(move || {
            println!("setting up system");
            f(context_clone);
        });

        // run blocking stream to keep system alive
        let (_, system_receiver) = channel::<Envelope>(8);
        Dispatcher::handle_stream_blocking(system_receiver, move |_| {});
    }

    pub fn register_actor(
        &mut self,
        actor: Arc<Mutex<Actor + Send>>,
        context: Context,
    ) -> ActorAddress {
        self.actor_id_counter += 1;
        self.actors
            .insert(self.actor_id_counter.clone(), actor.clone());

        let (sender, receiver) = channel::<Envelope>(16);

        let address = ActorAddress {
            id: self.actor_id_counter.clone(),
            sender: sender,
        };

        Dispatcher::handle_stream_background(receiver, move |envelope| {
            actor
                .lock()
                .unwrap()
                .handle(envelope.message, context.clone());
        });
        address
    }
}

#[derive(Clone)]
pub struct Context {
    system: Arc<Mutex<Router>>,
}
impl Context {
    pub fn register_actor(&self, actor: Arc<Mutex<Actor + Send>>) -> ActorAddress {
        self.system
            .lock()
            .unwrap()
            .register_actor(actor, self.clone())
    }
}
