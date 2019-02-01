#![deny(warnings)]
extern crate mpi;
extern crate hyper;
extern crate tokio;
extern crate futures;

use futures::*;
use std::io;

use mpi::traits::*;

//use std::io::{self, Write};
// use hyper::Client;
// use hyper::rt::{self, Future};

fn main() {
    let universe = mpi::initialize().unwrap();
    let processor = mpi::environment::processor_name().unwrap();
    let world = universe.world();
    let rank = world.rank();

    println!(
        "Hello from Host : {}, process {} of {}! Starting Request..",
        processor,
        world.rank(),
        world.size()
    );


    if rank == 0 {
        enter_master_routine(& universe);
    } else {
        enter_child_routine(& universe);
    }
    

    // rt::run(rt::lazy(move || {
    //     // This is main future that the runtime will execute.
    //     //
    //     // The `lazy` is because we don't want any of this executing *right now*,
    //     // but rather once the runtime has started up all its resources.
    //     //
    //     // This is where we will setup our HTTP client requests.


    //     let client = Client::new();

    //     let uri = "http://httpbin.org/ip".parse().unwrap();

    //     client
    //         .get(uri)
    //         .map(move |res| {
    //             println!("Response from Host({}), Process({}): HTTP Response {}", processor, rank, res.status());
    //         })
    //         .map_err(|err| {
    //             println!("Error: {}", err);
    //         })
    // }));
    


}

fn enter_child_routine<'a>(universe: &'a mpi::environment::Universe) {

    // ich kriege das universum nicht in den tokio spawn!
    // halp

    tokio::run(futures::lazy(move || {

        tokio::spawn(lazy(move || {

            println!("hello from {}", universe.world().rank());

            Ok(())

        }))
        
    }));


}

/**
 * 
 */
fn enter_master_routine(universe: &mpi::environment::Universe) {

    println!(
        "{} processes under control!", 
        universe.world().size() - 1
    );

    loop {
        let input: String = get_input("please enter a command");
        match input.as_ref() {
            "hallo" => {
                universe.world().process_at_rank(1).send(&2u64);
                println!("Hallo");
            }
            _ => {
                println!("command not recognized");
            }
        }
    }

}

/**
 * 
 */
pub fn get_input(prompt: &str) -> String{
    println!("{}",prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}