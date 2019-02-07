extern crate actor_model;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::message::Message;
use crate::application::message_serialization::*;
use crate::application::tcp_connection::TcpConnection;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct TcpListenActor {
    context: Option<Context<Message>>,
    port: u32,
    connections: Vec<TcpConnection>,
    master_addr: Option<Address<Message>>,
}
impl TcpListenActor {
    pub fn new(port: u32) -> TcpListenActor {
        TcpListenActor {
            context: None,
            port: port,
            connections: Vec::new(),
            master_addr: None,
        }
    }

    fn listen_for_tcp(&mut self) {
        println!("master listening on port {}", self.port);

        let ctx = self.context.as_ref().expect("unwrapping context");

        let port = self.port;
        let own_addr = ctx.own_address.clone();

        thread::spawn(move || {
            let addr = format!("localhost:{}", port);
            let listener = TcpListener::bind(addr).unwrap();
            // accept connections and process them, spawning a new thread for each one
            for stream in listener.incoming() {
                println!("new incoming tcp stream");
                match stream {
                    Ok(stream) => {
                        match stream.try_clone() {
                            Ok(stream_clone) => {
                                println!("cloned tcp stream");
                                // // self.connections.insert(0, s);
                                // sender.send(stream_clone).map_err(|error| {
                                //     println!("error sending TcpStream: {}", error)
                                // });
                                own_addr.send_self(Message::IncomingTcpConnection(TcpConnection(
                                    stream_clone,
                                )));
                                println!("clone was sent through channel");
                                let own_addr_clone = own_addr.clone();
                                handle_connection(stream, own_addr_clone);
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
        });
    }
}

fn handle_connection(stream: TcpStream, addr: Address<Message>) {
    thread::spawn(move || {
        println!("spawned new thread handling tcp stream");
        handle_messages(stream, |message: &str| {
            // println!("received tcp message, going to send actor message");
            addr.send_self(Message::IncomingTcpMessage(message.to_owned()));
        });
    });
}

impl Actor<Message> for TcpListenActor {
    fn handle(&mut self, message: Message, origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");

        match message {
            Message::IncomingTcpConnection(connection) => {
                // println!("TcpListenActor received IncomingTcpConnection");
                self.connections.push(connection);
            }
            Message::StartListenForTcp(master_addr) => {
                println!("TcpListenActor received StartListenForTcp");
                self.master_addr = Some(master_addr);
                self.listen_for_tcp();
            }
            Message::IncomingTcpMessage(str_message) => {
                // println!("TcpListenActor received IncomingTcpMessage");
                match parse_message(str_message.as_ref()) {
                    Some(actor_message) => {
                        // println!("received tcp message: {}", str_message);
                        match self.master_addr.as_ref() {
                            Some(addr) => ctx.send(&addr, actor_message),
                            None => println!("TcpActor has no master address to report to"),
                        };
                    }
                    None => (),
                };
            }
            Message::SendTcpMessage(actor_message) => {
                println!("TcpListenActor received SendTcpMessage");
                let message_str = serialize_message(*actor_message);
                let message_bytes = message_str.as_bytes();
                for stream in self.connections.iter_mut() {
                    stream.0.write(message_bytes);
                    stream.0.flush().unwrap();
                }
                //     Some(mut stream) => {
                //         stream.write(serialize_message(*actor_message).as_bytes());
                //         stream.flush().unwrap();
                //     }
                //     None => println!("unknown target"),
                // };
            }
            Message::ConnectToMaster(master_addr) => {
                println!("TcpListenActor received ConnectToMaster");
                self.master_addr = Some(master_addr);
                match TcpStream::connect("localhost:3333") {
                    Ok(stream) => {
                        // self.connections.insert(0, stream);
                        let own_addr_clone = self
                            .context
                            .as_ref()
                            .expect("unwrap context")
                            .own_address
                            .clone();
                        match stream.try_clone() {
                            Ok(stream_clone) => {
                                self.connections.push(TcpConnection(stream_clone));
                                handle_connection(stream, own_addr_clone);
                            }
                            Err(error) => println!("could not clone tcp stream: {}", error),
                        }
                    }
                    Err(e) => {
                        println!("Failed to connect: {}", e);
                    }
                };
            }
            _ => println!("TcpListenActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        println!("init TcpListenActor");

        self.context = Some(context);
    }
}

pub fn handle_messages<T>(mut conn: TcpStream, mut f: T)
where
    T: FnMut(&str),
{
    loop {
        // TODO: shut down stream when EOF is read
        let mut buffer = [0; 512];
        // let mut vec = Vec::new();
        conn.read(&mut buffer).unwrap();
        let message_as_string = String::from_utf8_lossy(&buffer[..]);
        let message_removed_nulls = message_as_string.trim_right_matches(char::from(0));
        //let message_removed_nulls = String::from_utf8(buffer[..].to_vec());

        // TODO: Wow, this looks weird.
        let message_to_pass_on = &(*message_removed_nulls);
        f(message_to_pass_on);
        // f(message_removed_nulls.unwrap().as_ref());
    }
}
