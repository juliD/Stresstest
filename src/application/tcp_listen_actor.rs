extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::message::Message;
use crate::application::message_serialization::*;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub struct TcpListenActor {
    context: Option<Context<Message>>,
    port: u32,
    connections: HashMap<u32, TcpStream>,
    connections_id_counter: u32,
}
impl TcpListenActor {
    pub fn new(port: u32) -> TcpListenActor {
        TcpListenActor {
            context: None,
            port: port,
            connections: HashMap::new(),
            connections_id_counter: 0,
        }
    }
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
                    match stream.try_clone() {
                        Ok(s) => {
                            self.connections.insert(0, s);
                            thread::spawn(move || {
                                handle_messages(stream, |message: &str| {
                                    match parse_message(message) {
                                        Some(actor_message) => {
                                            paddr.send(actor_message, Some(paddr.clone()))
                                        }
                                        None => {
                                            println!("could not parse tcp message: {}", message)
                                        }
                                    };
                                });
                            });
                        }
                        Err(error) => println!("could not clone tcp stream: {}", error),
                    };
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
