mod examples;
mod application;

extern crate num_cpus;

fn main() {
    
    // examples::split_computation::run();
    // examples::spawning_child_actors::run();
    // examples::tcp_stream::run();
    //examples::spawn_on_tcp_message::run();
    
    

    application::master::run();
}
