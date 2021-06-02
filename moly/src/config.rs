use confy;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server_addr: String,
    pub server_port: u16,
    pub client_port: u16,
    pub client_name: String,
    pub host_port: u16,
    pub host_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_addr: String::new(),
            server_port: 1324,
            client_port: 1234,
            client_name: String::new(),
            host_port: 1235,
            host_name: String::new()
        }
    }
}

pub fn load_or_default() -> Config {
    match confy::load("moly") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            Config::default()
        }
    }
}

pub fn save(config: Config) {
    confy::store("moly", config).expect("Saving config file");
}
