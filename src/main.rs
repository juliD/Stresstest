#[macro_use]
extern crate lazy_static;

mod application;
mod examples;
fn main() {
    let settings = application::read_config::parse_config();
    application::read_config::watch();
    //examples::split_computation::run();
    // examples::spawning_child_actors::run();
}
