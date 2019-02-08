extern crate config;
extern crate notify;
extern crate actor_model;

use actor_model::actor::*;
use actor_model::address::*;
use actor_model::context::*;
use crate::application::message::Message;

use std::collections::HashMap;
use config::*;
use std::sync::RwLock;
use notify::{RecommendedWatcher, DebouncedEvent, Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::str::FromStr;
use std::net::{SocketAddr};

const CONFIG_FILE_NAME: &str = "Settings";

const CONFIG_KEY_HOSTS: &str = "hosts";
const CONFIG_KEY_MASTER_SOCKET: &str = "master_socket";

const CONFIG_SOCKET_DELIMITER: &str = ",";

#[derive(Clone)]
pub struct AppConfig {
    master: SocketAddr,
    hosts: Vec<SocketAddr>,
}

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut settings = Config::default();
        settings.merge(File::with_name(CONFIG_FILE_NAME)).unwrap();

        settings
    });

}


pub struct ConfigActor{
    config: AppConfig,
    context: Option<Context<Message>>,
    master_addr: Option<Address<Message>>,
}

impl ConfigActor{
    pub fn new() -> ConfigActor{
        ConfigActor{
            config: parse_config(),
            context: None,
            master_addr: None,
        }
    }
    pub fn watch_config(&mut self){
        //channel to receive events
        let (tx, rx) = channel();

        //create watcher
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

        //add path to be watched. 
        watcher.watch("./Settings.toml", RecursiveMode::NonRecursive).unwrap();
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Write(_)) => {
                    println!(" * Settings.toml written; refreshing configuration ...");
                    SETTINGS.write().unwrap().refresh().unwrap();
                    self.config = parse_config();

                }

                Err(e) => println!("watch error: {:?}", e),

                _ => {
                    // Ignore event
                }
            }
        }
    }
}

impl Actor<Message> for ConfigActor{
    fn handle(&mut self, message: Message, _origin_address: Option<Address<Message>>) {
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");

        match message{
            Message::StartWatchingConfig(master_addr) => {
                println!("ConfigActor received StartWatchingConfig");
                self.master_addr = Some(master_addr);
                
                match self.master_addr.as_ref() {
                    Some(addr) => ctx.send(&addr, Message::Config(self.config.clone())),
                    None => println!("ConfigActor has no master address to report to"),
                };
                self.watch_config();
            }
            _ => println!("ConfigActor received unknown message"),
        }
    }
    fn start(&mut self, context: Context<Message>) {
        println!("init ConfigActor");

        self.context = Some(context);
        let ctx: &Context<Message> = self.context.as_ref().expect("unwrapping context");
        match self.master_addr.as_ref() {
            Some(addr) => ctx.send(&addr, Message::Config(self.config.clone())),
            None => println!("ConfigActor has no master address to report to"),
        };
    }
}


fn parse_config() -> AppConfig {
    let settings_hash = SETTINGS
        .read()
        .unwrap()
        .clone()
        .try_into::<HashMap<String, String>>()
        .unwrap();
        
    let config_keys: Vec<&str> = vec![CONFIG_KEY_HOSTS, CONFIG_KEY_MASTER_SOCKET];


    //check if all necessary keys are contained in config file
    match check_settings_exist(config_keys, &settings_hash){
        Ok(()) => println!("Config file complete"),
        Err(n) => panic!("Error: Key: {} is missing",n)
        //TODO: autofill with defaults?
    }

    let socket_addresses_raw_split = settings_hash
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
    
    let master_address = parse_socket_address_from_hashmap(&settings_hash, CONFIG_KEY_MASTER_SOCKET).expect("could not parse target socket address");

    
    AppConfig {
        hosts: hosts,
        master: master_address,
    }
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


fn check_settings_exist(settings_keys: Vec<&str>, settings_hashmap: &HashMap<String, String>) -> Result<(), String> {
    for key in settings_keys {
        if !settings_hashmap.contains_key(key) {
            return Err(key.to_string());
        }
    }
    Ok(())
}