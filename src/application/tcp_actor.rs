extern crate actor_model;
extern crate bufstream;
extern crate byteorder;
extern crate lazy_static;
extern crate reqwest;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;

use crate::application::config_actor::AppConfig;
use crate::application::message::Message;
use crate::application::message_serialization::*;
use crate::application::tcp_connection::TcpConnection;

use bufstream::*;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

pub struct TcpActor {
    context: Option<Context<Message>>,
    port: u32,
    connections: HashMap<u32, TcpConnection>,
    connection_id_counter: u32,
    master_addr: Option<Address<Message>>,
    config: Option<AppConfig>,
}
impl TcpActor {
    pub fn new(port: u32) -> TcpActor {
        TcpActor {
            context: None,
            port: port,
            connections: HashMap::new(),
            connection_id_counter: 0,
            master_addr: None,
            config: None,
        }
    }

    fn get_new_connection_id(&mut self) -> u32 {
        self.connection_id_counter += 1;
        self.connection_id_counter
    }

    fn listen_for_tcp(&mut self) {
        println!("master listening on port {}", self.port);

        let ctx = self.context.as_ref().expect("unwrapping context");

        let own_addr = ctx.own_address.clone();
        let master = self
            .config
            .as_ref()
            .expect("unwrapping config")
            .master
            .clone();
        thread::spawn(move || {
            let addr = format!("{}", master);
            let listener = TcpListener::bind(addr).unwrap();
            // accept connections and process them, spawning a new thread for each one
            for stream in listener.incoming() {
                println!("new incoming tcp stream");
                match stream {
                    Ok(stream) => {
                        match stream.try_clone() {
                            Ok(stream_clone) => {
                                println!("cloned tcp stream");
                                own_addr.send_self(Message::IncomingTcpConnection(TcpConnection(
                                    stream_clone,
                                )));
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

impl Actor<Message> for TcpActor {
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");

        match message {
            Message::IncomingTcpConnection(connection) => {
                let own_addr = ctx.own_address.clone();
                let conn_id = self.get_new_connection_id();
                self.connections.insert(conn_id, connection.clone());
                handle_connection(connection, own_addr, conn_id);
            }
            Message::StartListenForTcp(master_addr) => {
                self.master_addr = Some(master_addr);
                self.listen_for_tcp();
            }
            Message::IncomingTcpMessage(str_message) => {
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
                let message_str = serialize_message(*actor_message);
                let message_bytes = message_str.as_bytes();
                // println!("  writing message to {} streams...", self.connections.len());
                for connection in self.connections.values_mut() {
                    connection
                        .0
                        .write(message_bytes)
                        .map_err(|e| println!("{}", e));
                    connection.0.flush().unwrap();
                }
                // println!("  done writing");
            }
            Message::ConnectToMaster(master_addr) => {
                self.master_addr = Some(master_addr);

                let config = self.config.as_ref().expect("unwrapping config");
                match TcpStream::connect(config.master.clone()) {
                    Ok(stream) => {
                        let connection = TcpConnection(stream);
                        let own_addr = ctx.own_address.clone();
                        let conn_id = self.get_new_connection_id();
                        self.connections.insert(conn_id, connection.clone());
                        handle_connection(connection, own_addr, conn_id);
                    }
                    Err(e) => {
                        println!("Failed to connect: {}", e);
                    }
                };
            }
            Message::Config(config) => {
                self.config = Some(config);
            }
            Message::StreamDisconnected(connection_id) => {
                self.connections.remove(&connection_id);
            }
            _ => println!("TcpActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        self.context = Some(context);
    }
}

fn handle_connection(connection: TcpConnection, addr: Address<Message>, connection_id: u32) {
    thread::spawn(move || {
        println!("spawned new thread handling tcp stream");
        handle_messages(
            connection,
            |message: &str| {
                // println!("received tcp message, going to send actor message");
                addr.send_self(Message::IncomingTcpMessage(message.to_owned()));
            },
            || {
                println!("closing stream");
                addr.send_self(Message::StreamDisconnected(connection_id));
            },
        );
    });
}

pub fn handle_messages<T, Q>(
    connection: TcpConnection,
    mut handle_message: T,
    mut handle_disconnect: Q,
) where
    T: FnMut(&str),
    Q: FnMut(),
{
    let mut reader = BufStream::new(connection.0);
    let mut input: String = String::new();

    loop {
        input.clear();
        let input_size = match (&mut reader).read_line(&mut input) {
            Ok(s) => {
                // println!("read tcp msg of size {}: {}", s, input);
                s
            }
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        if input_size == 0 {
            break;
        };

        handle_message(input.clone().as_ref());
    }
    match reader.into_inner() {
        Ok(s) => {
            s.shutdown(Shutdown::Both).map_err(|e| println!("{}", e));
        }
        Err(e) => println!("{}", e),
    }
    handle_disconnect();
}
