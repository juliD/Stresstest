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
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");

    println!("init");
    ActorSystem::start(move || {
        let input_actor = ActorSystem::register_actor(InputActor { context: None }, None);
        let tcp_actor = ActorSystem::register_actor(TcpActor::new(3333), None);
        let config_actor = ActorSystem::register_actor(ConfigActor::new(tcp_actor.clone()), None);
        ActorSystem::register_actor(
            MasterActor::new(is_master_bool, tcp_actor, Some(input_actor), config_actor),
            None,
        );
    });
}
