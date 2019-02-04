extern crate actor_model;
extern crate tokio;
extern crate hyper;

use std::collections::LinkedList;
use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::actor_system::*;

use std::io::{self, BufRead};
use hyper::Client;
use hyper::rt::{self, Future};

struct WorkerActor{
    id: u32,
    context: Option<Context<String>>,
    target: hyper::Uri,
    status: bool,
}
impl WorkerActor{

}

impl Actor<String> for WorkerActor{
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>){
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();

        match vec[0]{
            "start" =>{
                if self.status {
                    self.status = false;
                    return;
                }
                let ctx = self.context.as_ref().expect("");
                let paddr = ctx.parent_address.as_ref().expect("").clone();
                let own_addr = ctx.own_address.clone();
                
                rt::spawn(rt::lazy(move || {
                    let client = Client::new();
                    let uri = "http://httpbin.org/ip".parse().unwrap();
                    client
                        .get(uri)
                        .map(move |res| {
                            println!("HTTP Response {}", res.status());
                            paddr.send("1".to_owned(), Some(own_addr));                    
                               
                            own_addr.send("start".to_owned(), Some(own_addr));
                        })
                        .map_err(|err| {
                            println!("Error: {}", err);
                        })
                }));                    
                
            }
            "target" =>{
                self.target = vec[1].parse().unwrap();
            }
            "stop" =>{
                println!("Stopped");
                self.status = true;
            }
            _=>{

            }
        }
    }
    fn start(&mut self, context: Context<String>){
        self.context = Some(context);
    }
}

struct LogActor{
    id: u32,
    context: Option<Context<String>>,
}

impl Actor<String> for LogActor{

    fn handle(&mut self, message: String, origin_address: Option<Address<String>>){

    }

    fn start(&mut self, context: Context<String>){
        self.context = Some(context);
    }
}

struct MasterActor{
    children: LinkedList<Address<String>>,
    child_id_counter: u32,
    context: Option<Context<String>>,
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

fn broadcast_children(ctx: &Context<String>, list: &LinkedList<Address<String>>, count: u32, message: String){
    let mut iter = list.iter();
    iter.next();

    for _ in 0..count-1 {
        match iter.next(){
            Some(c) =>{
                ctx.send(
                    &c,
                    message.to_owned()
                );
                
            },
            _ =>{

            }
        }

    }
}

impl Actor<String> for MasterActor{
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>){
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();
        let ctx: &Context<String> = self.context.as_ref().expect("");
        if vec[0] == "input" {
            
            match vec[1] {
                "start" => {
                    //TODO: send all worker children start message
                    broadcast_children(ctx, &self.children, self.child_id_counter, "start".to_string());
                    
                }
                "stop" => {
                    broadcast_children(ctx, &self.children, self.child_id_counter, "stop".to_string());
                }
                "log" =>{

                }
                "target" =>{
                    //TODO:check if target correct
                    broadcast_children(ctx, &self.children, self.child_id_counter, message);
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
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init MasterActor");



        //only if master process

        //Input Actor
        self.child_id_counter += 1;
        let ctx: &Context<String> = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(InputActor {
            id: self.child_id_counter,
            context: None,
        });
        self.children.push_back(child_addr.clone());
        ctx.send(&child_addr, "watch input".to_owned());
        

        
        //TODO: LogActor
        // self.child_id_counter += 1;
        // let ctx: &Context = self.context.as_ref().expect("");
        // let child_addr = ctx.register_actor(LogActor {
        //     id: self.child_id_counter,
        //     context: None,
        // });
        // self.children.push_front(child_addr.clone());

        //end if master process


        //create as many actors as cores available
        let num = num_cpus::get();
        for x in 0..num {
            self.child_id_counter += 1;
            let ctx: &Context<String> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(WorkerActor {
                        id: self.child_id_counter,
                        context: None,
                        target: "http://httpbin.org/ip".parse().unwrap(),
                        status: false,
                    });
            self.children.push_back(child_addr.clone());
        }

    }

    
}

struct InputActor {
    id: u32,
    context: Option<Context<String>>,
}
impl Actor<String> for InputActor{
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {
        if message == "watch input"{
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                
                match line{
                    Ok(input) =>{
                        let ctx: &Context<String> = self.context.as_ref().expect("");
                        let paddr: &Address<String> = ctx.parent_address.as_ref().expect("");
                        let answer = format!("input {}", input);
                        ctx.send(&paddr, answer.to_owned());
                        
                    }
                    Err(e) => println!("error: {}", e),
                }
                
            }

        }
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init InputActor");
    }
}



pub fn run() {
    println!("init");
    ActorSystem::start(|| {
        let spawning_addr = ActorSystem::register_actor(MasterActor::new(), None);
        
    });
}
