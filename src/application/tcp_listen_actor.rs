extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor::*;
use actor_model::actor_system::*;
use actor_model::address::*;
use actor_model::context::*;
use actor_model::message::*;
use std::collections::LinkedList;

use crate::application::address_parsing::*;
use crate::application::worker_actor::WorkerActor;
use crate::application::input_actor::InputActor;
use crate::application::message::Message;
use crate::application::utils::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::env;
use std::io::{self, BufRead};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

pub struct TcpListenActor {
    pub context: Option<Context<Message>>,
    pub port: u32,
}
impl Actor<Message> for TcpListenActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let addr = format!("localhost:{}", self.port);
        let listener = TcpListener::bind(addr).unwrap();

        // accept connections and process them, spawning a new thread for each one
        println!("master listening on port {}", self.port);
        for stream in listener.incoming() {
            let ctx = self.context.as_ref().expect("");
            let paddr = ctx.parent_address.as_ref().expect("").clone();
            match stream {
                Ok(stream) => {
                    //println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(move || {
                        handle_messages(stream, |message: &str| {
                            match parse_string_message(message) {
                                Some(actor_message) => {
                                    paddr.send(actor_message, Some(paddr.clone()))
                                }
                                None => println!("could not parse tcp message: {}", message),
                            };
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
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
        println!("init TcpListenActor");
    }
}


pub fn handle_messages<T>(mut conn: TcpStream, f: T)
where
    T: FnOnce(&str),
{
    let mut buffer = [0; 512];
    conn.read(&mut buffer).unwrap();
    let message_as_string = String::from_utf8_lossy(&buffer[..]);
    let message_removed_nulls = message_as_string.trim_right_matches(char::from(0));
    // TODO: Wow, this looks weird.
    let message_to_pass_on = &(*message_removed_nulls);
    f(message_to_pass_on);

    let string_response = "Thanks!";
    conn.write(string_response.as_bytes()).unwrap();
    conn.flush().unwrap();
}