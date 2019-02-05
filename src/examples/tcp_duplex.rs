use std::env;
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

  if is_master_bool {
    // master is connecting to slave connections

    match TcpStream::connect("localhost:3333") {
      Ok(mut stream) => {
        println!("connected to slave on port 3333");

        let msg = b"Hello!";

        stream.write(msg).unwrap();
        println!("Sent Hello, awaiting reply...");

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
      }
    }
    println!("Terminated.");
  } else {
    // slave is waiting for connections

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("slave listening on port 3333");
    for stream in listener.incoming() {
      match stream {
        Ok(stream) => {
          println!("New connection: {}", stream.peer_addr().unwrap());
          thread::spawn(move || {
            // connection succeeded
            handle_master(stream)
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

fn handle_master(mut stream: TcpStream) {
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
}
