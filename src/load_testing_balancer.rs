extern crate futures;
extern crate tokio;
extern crate tokio_serde_json;
extern crate config;
#[macro_use]
extern crate serde_json;

use futures::{Future, Sink};
use std::collections::HashMap;
use std::net::{SocketAddr};
use tokio::{
    codec::{FramedWrite, LengthDelimitedCodec},
    net::TcpStream,
};

use tokio_serde_json::WriteJson;

const CONFIG_FILE_NAME: &str = "Settings"; // without .toml ending

const CONFIG_KEY_HOSTS: &str = "hosts";
const CONFIG_KEY_TARGET: &str = "target";
const CONFIG_KEY_MASTER_SOCKET: &str = "master_socket";
const CONFIG_KEY_REQUESTS: &str = "requests";

const CONFIG_SOCKET_DELIMITER: &str = ",";

pub fn main() {
    let config = load_app_config();
    println!("{:?}", config);

    // Bind a server socket
    let socket = TcpStream::connect(&config.master);

    tokio::run(
        socket
            .and_then(move |socket| {
                // Delimit frames using a length header
                let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

                // Serialize frames with JSON
                let serialized = WriteJson::new(length_delimited);

                // Send the value
                serialized
                    .send(json!({
                        CONFIG_KEY_TARGET: config.target,
                        CONFIG_KEY_REQUESTS: config.requests,
                    })).map(|_| ())
            }).map_err(|_| ()),
    );
}

#[derive(Debug)]
struct AppConfig {
    master: SocketAddr,
    hosts: Vec<SocketAddr>,
    target: SocketAddr,
    requests: u64
}

fn load_app_config() -> AppConfig {
    let settings_hashmap = load_config_file();
    // TODO: move into constant?
    let config_keys: Vec<&str> = vec![CONFIG_KEY_HOSTS];

    if !check_settings_exist(config_keys, &settings_hashmap) {
        // TODO: better way of error handling?
        panic!("incomplete settings file");
    }
    // TODO: better error handling
    let socket_addresses_raw_split = settings_hashmap
        .get(CONFIG_KEY_HOSTS)
        .unwrap()
        .split(CONFIG_SOCKET_DELIMITER);
    let mut hosts: Vec<SocketAddr> = vec![];
    for socket_address_raw in socket_addresses_raw_split {
        match parse_socket_address(socket_address_raw) {
            Some(parsed_address) => hosts.push(parsed_address),
            // TODO: Handle errors better?
            None => panic!("could not parse socket address {}", socket_address_raw)
        }
    }
    let target_address = parse_socket_address_from_hashmap(&settings_hashmap, CONFIG_KEY_TARGET).expect("could not parse target socket address");
    let master_address = parse_socket_address_from_hashmap(&settings_hashmap, CONFIG_KEY_MASTER_SOCKET).expect("could not parse target socket address");
    // TODO: move into function
    let requests_raw = settings_hashmap.get(CONFIG_KEY_REQUESTS).expect("could not find requests");
    let requests: u64 = requests_raw.parse().expect("could not parse requests");

    AppConfig {
        hosts: hosts,
        target: target_address,
        master: master_address,
        requests: requests,
    }
}

fn check_settings_exist(settings_keys: Vec<&str>, settings_hashmap: &HashMap<String, String>) -> bool {
    for key in settings_keys {
        if !settings_hashmap.contains_key(key) {
            return false;
        }
    }
    true
}

fn load_config_file() -> HashMap<String, String> {
    let mut settings = config::Config::default();
    // TODO: handle error
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name(CONFIG_FILE_NAME))
        .unwrap();
    // TODO: handle error
    let settings_as_hashmap = settings.try_into::<HashMap<String, String>>().unwrap();
    settings_as_hashmap
}

fn parse_socket_address(raw_address_and_port: &str) -> Option<SocketAddr> {
    let parsed_address = raw_address_and_port.parse::<SocketAddr>();
    // TODO: Is it okay to discard the error here?
    match parsed_address {
        Ok(address) => Some(address),
        Err(_) => None,
    }
}

fn parse_socket_address_from_hashmap(hashMap: &HashMap<String, String>, keyInHashMap: &str) -> Option<SocketAddr> {
    parse_socket_address(hashMap.get(keyInHashMap)?)
}