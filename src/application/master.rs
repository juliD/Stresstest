extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor_system::*;
use std::env;

use crate::application::input_actor::InputActor;
use crate::application::master_actor::MasterActor;
use crate::application::tcp_listen_actor::TcpListenActor;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");

    println!("init");
    ActorSystem::start(move || {
        let input_actor = ActorSystem::register_actor(InputActor { context: None }, None);
        let tcp_actor = ActorSystem::register_actor(TcpListenActor::new(3333), None);
        let master_actor = ActorSystem::register_actor(
            MasterActor::new(is_master_bool, tcp_actor, Some(input_actor)),
            None,
        );
    });
}
