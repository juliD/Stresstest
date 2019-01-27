#![deny(warnings)]
extern crate mpi;
extern crate hyper;

use mpi::traits::*;

//use std::io::{self, Write};
use hyper::Client;
use hyper::rt::{self, Future};

fn main() {
    let universe = mpi::initialize().unwrap();
    let processor = mpi::environment::processor_name().unwrap();
    let world = universe.world();

    println!(
        "Hello from Host : {}, process {} of {}! Starting Request..",
        processor,
        world.rank(),
        world.size()
    );

    let rank = world.rank();

    rt::run(rt::lazy(move || {
        // This is main future that the runtime will execute.
        //
        // The `lazy` is because we don't want any of this executing *right now*,
        // but rather once the runtime has started up all its resources.
        //
        // This is where we will setup our HTTP client requests.


        let client = Client::new();

        let uri = "http://httpbin.org/ip".parse().unwrap();

        client
            .get(uri)
            .map(move |res| {
                println!("Response from Host({}), Process({}): HTTP Response {}", processor, rank, res.status());
            })
            .map_err(|err| {
                println!("Error: {}", err);
            })
    }));
    


}