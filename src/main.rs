extern crate futures;
extern crate tokio;

use tokio::prelude::*;
use tokio::prelude::task::Task;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use std::thread;

struct CheckMailboxTask { mailbox: Arc<Mutex<Mailbox>> }
impl CheckMailboxTask {
    fn run() {
        println!("running task");
    }
}
impl Future for CheckMailboxTask {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        let mut mailbox = self.mailbox.lock().unwrap();
        let task = futures::task::current();
        mailbox.set_task(task);
        let msg: u8 = mailbox.message.clone();
        if msg != 0 {
            println!("polled ready");
            Ok(Async::Ready(()))
        } else {
            println!("polled not ready");

            Ok(Async::NotReady)
        }
    }
}

type ActorAddress = u8;

trait Actor {}

struct Mailbox {
    actor_address: ActorAddress,
    message: u8,
    task: Arc<Mutex<Option<Task>>>
}
impl Mailbox {
    fn set_task(&mut self, task: Task) {
        let mut guard = self.task.lock().unwrap();
        *guard = Some(task);
    }
}

struct Router {
    actors: HashMap<ActorAddress, Box<Actor>>,
    mailboxes: HashMap<ActorAddress,  Arc<Mutex<Mailbox>>>,
    actor_id_counter: u8
}
impl Router {
    fn new() -> Router {
        Router {
             actors: HashMap::new(),
             mailboxes: HashMap::new(),
             actor_id_counter: 0 }
    }

    fn register<T: Actor + 'static>(&mut self, actor: T) -> ActorAddress {

        self.actor_id_counter += 1;
        self.actors.insert(self.actor_id_counter.clone(), Box::new(actor));

        let mailbox = Mailbox {
            actor_address: self.actor_id_counter.clone(),
            message: 0,
            task: Arc::new(Mutex::new(Option::None))
        };
        self.mailboxes.insert(self.actor_id_counter.clone(), Arc::new(Mutex::new(mailbox)));

        self.actor_id_counter.clone()
    }

    fn get_mailbox(&self, address: &ActorAddress) -> Option<Arc<Mutex<Mailbox>>> {
        match self.mailboxes.get(address) {
            Option::None => Option::None,
            Option::Some(m) => Option::Some(Arc::clone(m))
        }
    }
}

struct TestActor;
impl Actor for TestActor {}

fn main() {
    println!("init");

    let mut router: Router = Router::new();
    let actor: TestActor = TestActor { };
    let addr: ActorAddress = router.register(actor);
    let mailbox: Arc<Mutex<Mailbox>> = router.get_mailbox(&addr).unwrap();
    let mailbox_clone = Arc::clone(&mailbox);
    
    thread::spawn(move || {
        println!("child thread starts sleeping");
        sleep(Duration::from_millis(2000));
        println!("child thread wakes up");

        let mut unwrapped_mailbox = mailbox.lock().unwrap();
        let mut notified = false;
        {
            let wrapped_task: &Arc<Mutex<Option<Task>>> = &unwrapped_mailbox.task;
            let optional_task: MutexGuard<Option<Task>> = wrapped_task.lock().unwrap();
            if let Option::Some(ref t) = &*optional_task {
                println!("child thread notifying task");
                t.notify();
                notified = true;
            } else {
                println!("child thread has no task available");  
            }
        }
        unwrapped_mailbox.message = if notified { 1 } else { 0 };

        println!("child thread starts sleeping again");
        sleep(Duration::from_millis(2000));
        println!("child thread wakes up");
    });

    let task: CheckMailboxTask = CheckMailboxTask { mailbox: mailbox_clone };
    println!("starting tokio");
    tokio::run(task.and_then(|result| {
        println!("running and_then closure");
        futures::future::ok(())
    }));

    println!("done");
}
