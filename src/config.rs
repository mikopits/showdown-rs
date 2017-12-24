use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use toml;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default="default_host")]
    pub host: String,
    #[serde(default="default_port")]
    pub port: String,
    #[serde(default="default_mps")]
    pub throttle_ms: u64,
    #[serde(default="Default::default")]
    pub rooms: Vec<String>,
    #[serde(default="Default::default")]
    pub avatar: u64,
    #[serde(default="Default::default")]
    pub plugin_prefixes: Vec<String>,
    #[serde(default="Default::default")]
    pub case_insensitive: bool,
}

impl Config {
    pub fn new<P>(file_location: P) -> ::Result<Config>
        where P: AsRef<Path>,
    {
        let f = File::open(file_location)?;
        let mut br = BufReader::new(f);
        let mut contents = String::new();
        br.read_to_string(&mut contents)?;

        let decoded: Config = toml::from_str(&contents)?;

        Ok(decoded)
    }

    pub fn prefix_string(&self) -> String {
        "^(".to_string() + &self.plugin_prefixes.join("|") + ")"
    }
}

fn default_host() -> String { "sim.smogon.com".to_string() }
fn default_port() -> String { "8000".to_string() }
fn default_mps() -> u64 { 333 }
