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
    context: Option<Context>,
    target: hyper::Uri,
}
impl WorkerActor{

}

impl Actor for WorkerActor{
    fn handle(&mut self, message: String){
        let mut split = message.split(" ");
        let vec: Vec<&str> = split.collect();

        match vec[0]{
            "start" =>{
                let ctx: &Context = self.context.as_ref().expect("");
                let paddr: &Address = ctx.parent_address.as_ref().expect("");
                let own_addr: &Address = &ctx.own_address;
                for _ in 0..10{
                
                    rt::spawn(rt::lazy(move || {
                        let client = Client::new();
                        let uri = "http://httpbin.org/ip".parse().unwrap();
                        client
                            .get(uri)
                            .map(move |res| {
                                println!("HTTP Response {}", res.status());
                            })
                            .map_err(|err| {
                                println!("Error: {}", err);
                            })
                    }));                    
                    paddr.send("1".to_owned());                    
                }                
                own_addr.send("start".to_owned());
            }
            "target" =>{
                self.target = vec[1].parse().unwrap();
            }
            "stop" =>{
                println!("Stopped");
            }
            _=>{

            }
        }
    }
    fn receive_context(&mut self, context: Context){
        self.context = Some(context);
    }
}

struct LogActor{
    id: u32,
    context: Option<Context>,
}

impl Actor for LogActor{

    fn handle(&mut self, message: String){

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

fn broadcast_children(list: &LinkedList<Address>, count: u32, message: String){
    let mut iter = list.iter();
    iter.next();

    for _ in 0..count-1 {
        match iter.next(){
            Some(c) =>{
                c.send(message.to_owned());
            },
            _ =>{

            }
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
                    //TODO: send all worker children start message
                    broadcast_children(&self.children, self.child_id_counter, "start".to_string());
                    
                }
                "stop" => {
                    broadcast_children(&self.children, self.child_id_counter, "stop".to_string());
                }
                "log" =>{

                }
                "target" =>{
                    //TODO:check if target correct
                    broadcast_children(&self.children, self.child_id_counter, message);
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

        //Input Actor
        self.child_id_counter += 1;
        let ctx: &Context = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(InputActor {
            id: self.child_id_counter,
            context: None,
        });
        self.children.push_back(child_addr.clone());
        child_addr.send("watch input".to_owned());

        
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
            let ctx: &Context = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(WorkerActor {
                        id: self.child_id_counter,
                        context: None,
                        target: "http://httpbin.org/ip".parse().unwrap(),
                    });
            self.children.push_back(child_addr.clone());
        }

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
