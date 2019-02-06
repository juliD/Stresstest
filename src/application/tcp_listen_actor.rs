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

    fn listenForTcp(&self) {
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
        let addr = format!("localhost:{}", self.port);

        let ctx: &Context<Message> = self.context.as_ref().expect("");
        let parent_address = ctx.parent_address.unwrap().clone();

        match message {
            Message::IncomingTcpMessage(str_message) => {
                match parse_message(str_message.as_ref()) {
                    Some(actor_message) => ctx.send(&parent_address, actor_message),
                    None => (),
                };
            }
            Message::SendTcpMessage(target, actor_message) => match self.connections.get(&target) {
                Some(stream) => {
                    stream.write(serialize_message(*actor_message).as_bytes());
                }
                None => println!("unknown target"),
            },
        }
    }
    fn start(&mut self, context: Context<Message>) {
        println!("init TcpListenActor");

        self.context = Some(context);

        self.listenForTcp();
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
