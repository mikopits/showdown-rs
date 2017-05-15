//! A Rust chatbot for Pokemon Showdown.

#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate toml;
extern crate websocket;

// Crates for plugin mod
extern crate chrono;
extern crate csv;
extern crate rand;
extern crate rustc_serialize;
extern crate scoped_threadpool;

pub use self::bot::Bot;
pub use self::config::Config;
pub use self::error::{Error, Result};
pub use self::message::Message;
pub use self::plugin::Plugin;

pub mod plugin;
mod bot;
mod config;
mod error;
mod message;
mod target;

pub mod helpers {
    use regex::Regex;

    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"[^0-9a-zA-Z]").unwrap();
    }

    /// Removes non-alphanumeric characters from a string.
    /// We define non-alphanumeric as [^0-9a-zA-Z].
    ///
    /// Returns the string in lower case to guarantee uniqueness.
    pub fn sanitize(s: &str) -> String {
        REGEX.replace(s, "").into_owned().to_lowercase()
    }
}
