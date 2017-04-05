//! A Rust chatbot for Pokemon Showdown.

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate websocket;
extern crate time;
extern crate reqwest;
extern crate toml;
extern crate rustc_serialize;
#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate regex;
extern crate crossbeam;

pub mod bot;
pub mod plugin;
mod config;
mod message;
mod target;
