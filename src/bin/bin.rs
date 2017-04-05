extern crate showdown;

use showdown::bot::Bot;

#[cfg(not(test))]
fn main() {
    let mut b = Bot::new("config.toml");
    b.connect();
}
