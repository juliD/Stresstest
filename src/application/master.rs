extern crate actor_model;
extern crate reqwest;
extern crate byteorder;
use std::collections::LinkedList;
use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::actor_system::*;

use std::env;


use std::io::{self, BufRead};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

struct WorkerActor{
    id: u32,
    context: Option<Context<String>>,
    target: String,
    status: bool,
}
impl WorkerActor{

}

fn request() -> Result<(), Box<std::error::Error>>{
    let mut res = reqwest::get("https://www.rust-lang.org/")?;
    println!("Status = {}", res.status());

    Ok(())
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
                let paddr = ctx.parent_address.as_ref().expect("");
                let own_addr = ctx.own_address.clone();
                      
                request();           
                ctx.send(&paddr, "1".to_string());
                //ctx.send(&own_addr, "start".to_string());
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
    master: bool,
    started: bool,
    counter: u64,
}
impl MasterActor{
    fn new(is_master: bool) -> MasterActor {
        MasterActor {
            children: LinkedList::new(),
            child_id_counter: 0,
            context: None,
            master: is_master,
            started: false,
            counter: 0
        }
    }
    fn send_slaves(&mut self, message: String){
        match TcpStream::connect("localhost:3333") {
            Ok(mut stream) => {
                
                stream.write(message.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
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
        
        if self.master{
            match message.as_ref() {
                "1"=>{
                    self.counter+=1;
                    println!("counting up {}",self.counter);
                }
                "start" => {
                    
                    broadcast_children(ctx, &self.children, self.child_id_counter, "start".to_string());
                    self.send_slaves(message);
                    
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
                    println!("This is what I got: -{}-",message);
                    println!("Not a valid input. Write help to get a list of commands.")
                }
            }
        }
        else{
            
            if message.trim() == "start"{
                println!("Yaaaaaay -{}-",message.trim());
            }
            else{
                println!("Buuuhhh -{}-",message.trim());
            }
            // if self.started{
            //     broadcast_children(ctx, &self.children, self.child_id_counter, "stop".to_string());
            // }
            // else{
            //     broadcast_children(ctx, &self.children, self.child_id_counter, "start".to_string());
            // }
            // self.started = !self.started;

        }
        
        
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init MasterActor");


        let mut port = 3333;
        if self.master {
            //Input Actor
            self.child_id_counter += 1;
            let ctx: &Context<String> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(InputActor {
                id: self.child_id_counter,
                context: None,
            });
            self.children.push_back(child_addr.clone());
            ctx.send(&child_addr, "watch input".to_owned());
            port = 3001;
        }

        //create TCPListener
        self.child_id_counter += 1;
        let ctx: &Context<String> = self.context.as_ref().expect("");
        let child_addr = ctx.register_actor(TcpListenActor {
                    context: None,
                    port: port,
                });
        self.children.push_back(child_addr.clone());
        ctx.send(&child_addr, "listen".to_string());


        //create as many actors as cores available
        let num = num_cpus::get();
        for x in 0..num {
            self.child_id_counter += 1;
            let ctx: &Context<String> = self.context.as_ref().expect("");
            let child_addr = ctx.register_actor(WorkerActor {
                        id: self.child_id_counter,
                        context: None,
                        target: "http://httpbin.org/ip".to_owned(),
                        status: false,
                    });
            self.children.push_back(child_addr.clone());
        }

    }

    
}

struct TcpListenActor{
    context: Option<Context<String>>,
    port: u32,
}
impl Actor<String> for TcpListenActor{
    fn handle(&mut self, message: String, origin_address: Option<Address<String>>) {

        let addr = format!("localhost:{}",self.port);
        let listener = TcpListener::bind(addr).unwrap();

        
        // accept connections and process them, spawning a new thread for each one
        println!("master listening on port {}",self.port);
        for stream in listener.incoming() {
            let ctx = self.context.as_ref().expect("");
            let paddr = ctx.parent_address.as_ref().expect("").clone();
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || {                

                        handle_messages(stream, |message: &str|{
                            println!("received message {}", message);
                            paddr.send(message.to_string(), Some(paddr.clone()));
                        });
                    });
                }
                Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
                }
            }
        }
        
    }
    fn start(&mut self, context: Context<String>) {
        self.context = Some(context);
        println!("init InputActor");
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
                        
                        ctx.send(&paddr, input.to_owned());
                        
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
    let args: Vec<String> = env::args().collect();
    let is_master = &args[1];
    let is_master_bool: bool = is_master.parse().expect("master not parsable");


    println!("init");
    ActorSystem::start(move || {        
        let spawning_addr = ActorSystem::register_actor(MasterActor::new(is_master_bool), None);
        
        
    });
}

fn handle_messages<T>(mut conn: TcpStream, f: T)
where
  T: FnOnce(&str)
{
  let mut buffer = [0; 512];
  conn.read(&mut buffer).unwrap();
  let message_as_string = String::from_utf8_lossy(&buffer[..]);
  // TODO: Wow, this looks weird.
  let message_to_pass_on = &(*message_as_string);
  f(message_to_pass_on);

  let string_response = "Thanks!";
  conn.write(string_response.as_bytes()).unwrap();
  conn.flush().unwrap();
}