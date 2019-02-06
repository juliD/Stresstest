extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor_system::*;
use std::env;

use crate::application::master_actor::MasterActor;


pub fn run() {
    let args: Vec<String> = env::args().collect();
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");

    println!("init");
    ActorSystem::start(move || {
        ActorSystem::register_actor(MasterActor::new(is_master_bool), None);
    });
}
