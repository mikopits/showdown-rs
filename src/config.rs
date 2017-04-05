//! Deserializes the toml configuration file.

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use pretty_env_logger;
use toml;

/// A `Config` holds the configuration information supplied by the toml.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub user: String,
    pub pass: String,
    pub messages_per_ms: u64,
    pub rooms: Vec<String>,
    pub avatar: u64,
    pub plugin_prefixes: Vec<String>,
    pub case_sensitive: bool,
}

impl Config {
    /// Creates a new `Config` by deserializing toml.
    pub fn new(file_location: String) -> Self {
        let _ = pretty_env_logger::init();
        let f = File::open(file_location).expect("failed to read config file");
        let mut br = BufReader::new(f);
        let mut contents = String::new();
        br.read_to_string(&mut contents).expect("failed to write config file to buffer");

        let decoded: Config = match toml::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                error!("Could not decode toml: {:?}", e);
                panic!(e);
            }
        };
        decoded
    }
}
