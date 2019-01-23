extern crate config;
extern crate notify;



use std::collections::HashMap;
use config::*;
use std::sync::RwLock;

const CONFIG_FILE_NAME: &str = "Settings";

const CONFIG_KEY_HOSTS: &str = "hosts";
const CONFIG_KEY_TARGET: &str = "target";
const CONFIG_KEY_MASTER_SOCKET: &str = "master_socket";
const CONFIG_KEY_REQUESTS: &str = "requests";
const CONFIG_KEY_MILLISECONDS: &str = "interval_milliseconds";

const CONFIG_SOCKET_DELIMITER: &str = ",";

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut settings = Config::default();
        settings.merge(File::with_name(CONFIG_FILE_NAME)).unwrap();

        settings
    });
}

fn to_hashmap() {
    SETTINGS
        .read()
        .unwrap()
        .clone()
        .try_into::<HashMap<String, String>>()
        .unwrap();
}

pub fn parse_config() {
    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name(CONFIG_FILE_NAME)).unwrap();
    
    let settings_hash = SETTINGS.read().clone().try_into::<HashMap<String, String>>().unwrap();
    let config_keys: Vec<&str> = vec![CONFIG_KEY_HOSTS, CONFIG_KEY_TARGET, CONFIG_KEY_MASTER_SOCKET, CONFIG_KEY_REQUESTS, CONFIG_KEY_MILLISECONDS];

    //check if all necessary keys are contained in config file
    match check_settings_exist(config_keys, &settings_hash){
        Ok(()) => println!("Config file complete"),
        Err(n) => println!("Error: Key: {} is missing",n)
    }
}

fn check_settings_exist(settings_keys: Vec<&str>, settings_hashmap: &HashMap<String, String>) -> Result<(), String> {
    for key in settings_keys {
        if !settings_hashmap.contains_key(key) {
            return Err(key.to_string());
        }
    }
    Ok(())
}