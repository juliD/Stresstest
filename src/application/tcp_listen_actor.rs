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
    master_addr: Option<Address<Message>>,
}
impl TcpListenActor {
    pub fn new(port: u32) -> TcpListenActor {
        TcpListenActor {
            context: None,
            port: port,
            connections: HashMap::new(),
            connections_id_counter: 0,
            master_addr: None,
        }
    }

    fn listen_for_tcp(&mut self) {
        let addr = format!("localhost:{}", self.port);
        let listener = TcpListener::bind(addr).unwrap();
        // accept connections and process them, spawning a new thread for each one
        println!("master listening on port {}", self.port);
        for stream in listener.incoming() {
            let ctx = self.context.as_ref().expect("");
            let own_addr = ctx.own_address.clone();
            match stream {
                Ok(stream) => {
                    //println!("New connection: {}", stream.peer_addr().unwrap());
                    match stream.try_clone() {
                        Ok(s) => {
                            self.connections.insert(0, s);
                            thread::spawn(move || {
                                handle_messages(stream, |message: &str| {
                                    own_addr
                                        .send_self(Message::IncomingTcpMessage(message.to_owned()));
                                    // match parse_message(message) {
                                    //     Some(actor_message) => own_addr.send_self(Message::IncomingTcpMessage actor_message),
                                    //     None => {
                                    //         println!("could not parse tcp message: {}", message)
                                    //     }
                                    // };
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
}
impl Actor<Message> for TcpListenActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("");

        match message {
            Message::StartListenForTcp(master_addr) => {
                self.master_addr = Some(master_addr);
                self.listen_for_tcp();
            }
            Message::IncomingTcpMessage(str_message) => {
                match parse_message(str_message.as_ref()) {
                    Some(actor_message) => {
                        println!("received tcp message: {}", str_message);
                        match self.master_addr.as_ref() {
                            Some(addr) => ctx.send(&addr, actor_message),
                            None => println!("TcpActor has no master address to report to")
                        };
                    }
                    None => (),
                };
            }
            Message::SendTcpMessage(target, actor_message) => match self.connections.get(&target) {
                Some(mut stream) => {
                    stream.write(serialize_message(*actor_message).as_bytes());
                }
                None => println!("unknown target"),
            },
            _ => println!("MasterActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        println!("init TcpListenActor");

        self.context = Some(context);
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
