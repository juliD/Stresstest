extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor_system::*;
use std::env;

use crate::application::config_actor::ConfigActor;
use crate::application::input_actor::InputActor;
use crate::application::master_actor::MasterActor;
use crate::application::tcp_actor::TcpActor;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        run_as_slave();
    } else {
        let master_flag: String = args[1].parse().expect("Could not parse flag as intended. Maybe you meant --master?");
        if master_flag == "master" {
            run_as_master();
        } else {
            run_as_slave();
        }
    }
}

pub fn run_as_slave() {
    println!("Run system as slave");
    init_system(false);
}

pub fn run_as_master() {
    println!("Run system as master");
    init_system(true);
}

pub fn init_system(is_master: bool) {
    ActorSystem::start(move || {
        let input_actor = ActorSystem::register_actor(InputActor { context: None }, None);
        let tcp_actor = ActorSystem::register_actor(TcpActor::new(3333), None);
        let config_actor = ActorSystem::register_actor(ConfigActor::new(tcp_actor.clone()), None);
        ActorSystem::register_actor(
            MasterActor::new(is_master, tcp_actor, Some(input_actor), config_actor),
            None,
        );
    });
}
