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
use std::sync::mpsc::*;
use std::hash::{Hash, Hasher};

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

        println!("poll waits for lock of mailbox");
        let mut mailbox = self.mailbox.lock().unwrap();
        println!("poll enters lock of mailbox");
        let task = futures::task::current();
        mailbox.set_task(task);
        let message: Result<Message, TryRecvError> = mailbox.receiver.try_recv();
        let r = match message{
            Ok(msg) => {
                println!("polled ready");
                Ok(Async::Ready(()))
            }
            _ => {
                println!("polled not ready");
                Ok(Async::NotReady)
            }
        };
        println!("poll leaves lock of mailbox");
        r
    }
}

type Message = String;
type ActorId = u64;

struct ActorAddress {
    id: ActorId,
    sender: Sender<Message>
}
impl Hash for ActorAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for ActorAddress {
    fn eq(&self, other: &ActorAddress) -> bool {
        self.id == other.id
    }
}
impl Eq for ActorAddress {}

trait Actor {}

struct Mailbox {
    actor_id_counter: ActorId,
    receiver: Receiver<Message>,
    task: Arc<Mutex<Option<Task>>>
}
impl Mailbox {
    fn set_task(&mut self, task: Task) {
        let mut guard = self.task.lock().unwrap();
        *guard = Some(task);
    }
}

struct Router {
    actors: HashMap<ActorId, Box<Actor>>,
    mailboxes: HashMap<ActorId,  Arc<Mutex<Mailbox>>>,
    actor_id_counter: ActorId
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

        let (tx, rx) = channel();
        let mailbox = Mailbox {
            actor_id_counter: self.actor_id_counter.clone(),
            receiver: rx,
            task: Arc::new(Mutex::new(Option::None))
        };
        self.mailboxes.insert(self.actor_id_counter.clone(), Arc::new(Mutex::new(mailbox)));

        ActorAddress {id: self.actor_id_counter.clone(), sender: tx}
    }

    fn get_mailbox(&self, id: &ActorId) -> Option<Arc<Mutex<Mailbox>>> {
        match self.mailboxes.get(id) {
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
    let mailbox: Arc<Mutex<Mailbox>> = router.get_mailbox(&addr.id).unwrap();
    let mailbox_clone = Arc::clone(&mailbox);
    
    thread::spawn(move || {
        println!("child thread starts sleeping");
        sleep(Duration::from_millis(2000));
        println!("child thread wakes up");

        println!("child thread waits for lock of mailbox");
        let mut unwrapped_mailbox = mailbox.lock().unwrap();
        println!("child thread enters lock of mailbox");
        let mut notified = false;
        {
            let wrapped_task: &Arc<Mutex<Option<Task>>> = &unwrapped_mailbox.task;
            println!("child thread waits for lock of task");
            let optional_task: MutexGuard<Option<Task>> = wrapped_task.lock().unwrap();
            println!("child thread enters lock of task");
            if let Option::Some(ref t) = &*optional_task {
                println!("child thread notifying task");
                t.notify();
                notified = true;
            } else {
                println!("child thread has no task available");  
            }
            println!("child thread leaves lock of task");
        }
        addr.sender.send("testmessage".to_string());

        println!("child thread starts sleeping again");
        sleep(Duration::from_millis(2000));
        println!("child thread wakes up");

        println!("child thread leaves lock of mailbox");
    });

    let task: CheckMailboxTask = CheckMailboxTask { mailbox: mailbox_clone };
    println!("starting tokio");
    tokio::run(task.and_then(|result| {
        println!("running and_then closure");
        futures::future::ok(())
    }));

    println!("done");
}
