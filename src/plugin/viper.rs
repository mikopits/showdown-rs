extern crate chrono;
extern crate rand;
extern crate regex;

use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::OpenOptions;

use chrono::{DateTime, Duration, UTC};
use rand::Rng;
use regex::Regex;
use ::{Message, Plugin, helpers};

static FILE_PATH: &str = "data/viper.txt";

lazy_static! {
    static ref REGEX: Regex =
        Regex::new(r"^(>|#|uhh\s|le\s)vip(er|a)").unwrap();
}

#[derive(Debug)]
pub struct ViperPlugin {
    vipers: Vec<String>,
    last_used_map: HashMap<String, DateTime<UTC>>,
    banlist_map: HashMap<String, DateTime<UTC>>,
    cooldown: Duration,
    ban_duration: Duration
}

impl Plugin for ViperPlugin {
    fn new() -> Box<Plugin> {
        let file = OpenOptions::new()
            .read(true)
            .open(FILE_PATH)
            .expect("Failed to open file");

        let buf = BufReader::new(file);

        Box::new(ViperPlugin {
            vipers: buf.lines().map(|l| l.unwrap()).collect(),
            last_used_map: HashMap::new(),
            banlist_map: HashMap::new(),
            cooldown: Duration::seconds(60),
            ban_duration: Duration::minutes(10)
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        REGEX.is_match(&msg.payload)
    }

    fn handle(&mut self, msg: &Message) {
        let now = UTC::now();
        let user = helpers::sanitize(&msg.user.name);

        // Unban the user if their ban is over
        if self.banlist_map.contains_key(&user) {
            let ban_time = *self.banlist_map.get(&user).unwrap();
            if now.signed_duration_since(ban_time) >= self.ban_duration {
                self.banlist_map.remove(&user);
            } else {
                return
            }
        }

        // Ban the user if they used twice within one cooldown duration
        if self.last_used_map.contains_key(&user) {
            let last_used = *self.last_used_map.get(&user).unwrap();
            if now.signed_duration_since(last_used) < self.cooldown {
                self.banlist_map.insert(user, now);
                msg.send(self.ban_message(&msg.user.name));
                return
            }
        }

        self.last_used_map.insert(user, now);

        let viper = match rand::thread_rng().choose(&self.vipers) {
            Some(v) => v,
            None => {
                return msg.reply("Could not get a viper ugh =.= smh @ shy imouto")
            }
        };

        msg.reply(viper.to_owned());
    }
}

impl ViperPlugin {
    fn ban_message(&self, user: &str) -> String {
        let cooldown_secs = self.cooldown.num_seconds();
        let ban_mins = self.ban_duration.num_minutes();
        let notice = "**Kill Urself My Man**";
        format!("{} (Coward {} Can Only Handle 1 Vipa Per {} Seconds. \
        You'll Is Spendin' {} Minutes In Tha Pen (Penitentiary)",
        notice, user, cooldown_secs, ban_mins)
    }
}
