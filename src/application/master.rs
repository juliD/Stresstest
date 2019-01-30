extern crate actor_model;
extern crate tokio;
extern crate hyper;

use std::collections::LinkedList;
use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::router::*;

use std::io::{self, BufRead};
use hyper::Client;
use hyper::rt::{self, Future};

struct WorkerActor{
    id: u32,
    children: LinkedList<Address>,
    child_id_counter: u32,
    context: Option<Context>,
}
impl WorkerActor{

}

impl Actor for WorkerActor{
    fn handle(&mut self, message: String){
        
        match message.as_ref(){
            "start" =>{
                println!("I started");
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
                //             println!("ID: {} HTTP Response {}", self.id, res.status());
                //         })
                //         .map_err(|err| {
                //             println!("Error: {}", err);
                //         })
                // }));
            }
            "stop" =>{

            }
            _=>{

            }
        }
    }
    fn receive_context(&mut self, context: Context){
        self.context = Some(context);
    }
}

struct MasterActor{
    children: LinkedList<Address>,
    child_id_counter: u32,
    context: Option<Context>,
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

impl Actor for MasterActor{
    fn handle(&mut self, message: String){
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();
        if vec[0] == "input" {
            
            match vec[1] {
                "start" => {
                    self.child_id_counter += 1;
                    let ctx: &Context = self.context.as_ref().expect("");
                    let child_addr = ctx.register_actor(WorkerActor {
                        children: LinkedList::new(),
                        id: self.child_id_counter,
                        child_id_counter: 0,
                        context: None,
                    });
                    self.children.push_front(child_addr.clone());
                    child_addr.send("start".to_owned());
                }
                "stop" => {

                }
                "log" =>{

                }
                "target" =>{

                }
                "help" =>{
                    println!("--------------------------------------------------------------------------");
                    println!("target [target address]   specify the target which should be stress tested");
                    println!("start                     start stress test");
                    println!("stop                      stop stress test");
                    println!("log                       get current results");
                    println!("help                      get all possible commands");
                    println!("--------------------------------------------------------------------------");
                }
                _ =>{
                    println!("Not a valid input. Write help to get a list of commands.")
                }
            }
        }
    }
    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
        println!("init MasterActor");

        //only if master process
        self.child_id_counter += 1;
        let ctx: &Context = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(InputActor {
            id: self.child_id_counter,
            context: None,
        });
        self.children.push_front(child_addr.clone());
        child_addr.send("watch input".to_owned());

        //TODO:create LogActor

        //end if master process


    }
}

struct InputActor {
    id: u32,
    context: Option<Context>,
}
impl Actor for InputActor{
    fn handle(&mut self, message: String) {
        if message == "watch input"{
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                
                match line{
                    Ok(input) =>{
                        let ctx: &Context = self.context.as_ref().expect("");
                        let paddr: &Address = ctx.parent_address.as_ref().expect("");
                        let answer = format!("input {}", input);
                        paddr.send(answer.to_owned());
                    }
                    Err(e) => println!("error: {}", e),
                }
                
            }

        }
    }
    fn receive_context(&mut self, context: Context) {
        self.context = Some(context);
        println!("init InputActor");
    }
}

pub fn run() {
    println!("init");
    Router::start(|| {
        let spawning_addr = Router::register_actor(MasterActor::new(), None);
        
    });
}
