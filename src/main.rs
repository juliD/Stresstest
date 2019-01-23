#[macro_use]
extern crate lazy_static;

mod application;
mod examples;
fn main() {
    application::read_config::watch();
    //examples::split_computation::run();
    // examples::spawning_child_actors::run();
}
