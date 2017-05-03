extern crate showdown;
extern crate env_logger;

use showdown::{Bot, Plugin, plugin};

#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();

    let b = Bot::new("config.toml").unwrap();

    // Register plugins before connecting
    b.register(plugin::MemePlugin::new());
    b.register(plugin::ViperPlugin::new());

    b.connect().unwrap();
}
