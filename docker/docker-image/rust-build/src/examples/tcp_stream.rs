extern crate futures;
extern crate serde_json;
extern crate tokio;
extern crate tokio_serde_json;

use futures::{Future, Stream};
use tokio::{
    codec::{FramedRead, LengthDelimitedCodec},
    net::TcpListener,
};
use serde_json::Value;
use tokio_serde_json::ReadJson;

pub fn run() {
    // Bind a server socket
    let listener = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    println!("listening on {:?}", listener.local_addr());
    tokio::run(
        listener
            .incoming()
            .for_each(|socket| {
                // Delimit frames using a length header
                let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
                // Deserialize frames
                let deserialized = ReadJson::<_, Value>::new(length_delimited)
                    .map_err(|e| println!("ERR: {:?}", e));
                // Spawn a task that prints all received messages to STDOUT
                tokio::spawn(deserialized.for_each(|msg| {
                    println!("GOT: {:?}", msg);
                    println!("{:?}", msg.as_object().unwrap().get("root").unwrap().as_str().unwrap());
                    Ok(())
                }));
                Ok(())
            }).map_err(|_| ()),
    );
}