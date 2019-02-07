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
    connections: HashMap<u32, TcpStream>,
    connections_id_counter: u32,
    master_addr: Option<Address<Message>>,
    connection_receiver: Option<Receiver<TcpStream>>,
}
impl TcpListenActor {
    pub fn new(port: u32) -> TcpListenActor {
        TcpListenActor {
            context: None,
            port: port,
            connections: HashMap::new(),
            connections_id_counter: 0,
            master_addr: None,
            connection_receiver: None,
        }
    }

    fn listen_for_tcp(&mut self) {
        println!("master listening on port {}", self.port);

        let ctx = self.context.as_ref().expect("unwrapping context");

        let port = self.port;
        let own_addr = ctx.own_address.clone();

        let (sender, receiver) = channel::<TcpStream>();

        self.connection_receiver = Some(receiver);

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
            // Message::TryAcceptConnection => {
            //     match &self.connection_receiver {
            //         Some(recv) => {
            //             match recv.recv_timeout(Duration::from_millis(100)) {
            //                 Ok(mut stream) => {
            //                     println!("received stream");
            //                     stream.write("start".as_bytes());
            //                     self.connections.insert(0, stream);
            //                 }
            //                 Err(error) => (), //println!("{}", error)
            //             }
            //         }
            //         None => {
            //             println!("no message in receiver");
            //         }
            //     }
            //     ctx.send_self(Message::TryAcceptConnection);
            // }
            Message::IncomingTcpConnection(connection) => {
                // println!("TcpListenActor received IncomingTcpConnection");
                self.connections.insert(0, connection.0);
            }
            Message::StartListenForTcp(master_addr) => {
                println!("TcpListenActor received StartListenForTcp");
                self.master_addr = Some(master_addr);
                // ctx.send_self(Message::TryAcceptConnection);
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
            Message::SendTcpMessage(target, actor_message) => {
                println!("TcpListenActor received SendTcpMessage");
                match self.connections.get(&target) {
                    Some(mut stream) => {
                        stream.write(serialize_message(*actor_message).as_bytes());
                        stream.flush().unwrap();
                    }
                    None => println!("unknown target"),
                };
            }
            Message::ConnectToMaster(master_addr) => {
                println!("TcpListenActor received ConnectToMaster");
                self.master_addr = Some(master_addr);
                match TcpStream::connect("localhost:3333") {
                    Ok(mut stream) => {
                        // self.connections.insert(0, stream);
                        let own_addr_clone = self
                            .context
                            .as_ref()
                            .expect("unwrap context")
                            .own_address
                            .clone();
                        match stream.try_clone() {
                            Ok(stream_clone) => {
                                self.connections.insert(0, stream_clone);
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
