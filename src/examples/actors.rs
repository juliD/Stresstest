use std::collections::LinkedList;
use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::actor_system::*;

struct MasterActor{
    children: LinkedList<Address<String>>,
    child_id_counter: u32,
    context: Option<Context<String>>,
}

impl MasterActor{
    fn new() -> MasterActor {
        MasterActor {
            children: LinkedList::new(),
            child_id_counter: 0,
            context: None,
        }
    }
  
}

pub fn run() {
    println!("init");
    ActorSystem::start(|| {
        let spawning_addr = ActorSystem::register_actor(MasterActor::new(), None);
        
    });
}