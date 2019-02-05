// TODO: slave only has to listen to ONE master
// TODO: simplify master code and reuse listening from slave

extern crate actor_model;
extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::env;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;

pub fn run() {
  let args: Vec<String> = env::args().collect();

  let is_master = &args[1];
  let is_master_bool: bool = is_master.parse().expect("master not parsable");

  println!("running");
  println!("is master: {}", is_master_bool);

  let slave_ip = "127.0.0.1";
  let salve_port = 3001;
  let master_ip = "127.0.0.1";
  let master_port = 3000;

  let number_of_actors = "5";
  /*   let mut number_of_actors_bytes = vec![];
  number_of_actors_bytes.write_u32::<BigEndian>(number_of_actors).unwrap(); */

  if is_master_bool {
    // master is connecting to slave

    match TcpStream::connect("localhost:3333") {
      Ok(mut stream) => {
        /* println!("connected to slave on port 3333");

          let msg = b"Hello!";
          //let msg = &number_of_actors_bytes[..];

          stream.write(msg).unwrap();
          println!("Sent {}, awaiting reply...", number_of_actors);

          let mut data = [0 as u8; 6]; // using 6 byte buffer
          match stream.read_exact(&mut data) {
            Ok(_) => {
              if &data == msg {
                println!("Reply is ok!");
              } else {
                let text = from_utf8(&data).unwrap();
                println!("Unexpected reply: {}", text);
              }
            }
            Err(e) => {
              println!("Failed to receive data: {}", e);
            }
          }
        }
        Err(e) => {
          println!("Failed to connect: {}", e);
        } */
        stream.write(number_of_actors.as_bytes()).unwrap();
        stream.flush().unwrap();
      }
      Err(e) => {
        println!("Failed to connect: {}", e);
      }
    }
    println!("terminated");
  } else {
    // slave is waiting for connections

    let listener = TcpListener::bind("localhost:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("slave listening on port 3333");
    for stream in listener.incoming() {
      match stream {
        Ok(stream) => {
          println!("New connection: {}", stream.peer_addr().unwrap());
          thread::spawn(move || {
            // connection succeeded
            // handle_master(stream)
            handle_messages(stream, |message: &str|{
              println!("received message {}", message);
            });
          });
        }
        Err(e) => {
          println!("Error: {}", e);
          /* connection failed */
        }
      }
    }
    // close the socket server
    drop(listener);
  }
}

/* fn handle_master(mut stream: TcpStream) {
  let mut data = [0 as u8; 50]; // using 50 byte buffer
  while match stream.read(&mut data) {
    Ok(size) => {
      // echo everything!
      stream.write(&data[0..size]).unwrap();
      true
    }
    Err(_) => {
      println!(
        "An error occurred, terminating connection with {}",
        stream.peer_addr().unwrap()
      );
      stream.shutdown(Shutdown::Both).unwrap();
      false
    }
  } {}
} */

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
