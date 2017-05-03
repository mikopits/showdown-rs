extern crate chrono;
extern crate csv;
extern crate rand;
extern crate regex;
extern crate scoped_threadpool;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use chrono::{DateTime, Duration, UTC};
use rand::Rng;
use regex::{Regex, RegexBuilder};
use scoped_threadpool::Pool;
use ::{Message, Plugin, helpers};

static FILE_PATH: &str = "data/memes.csv";

lazy_static! {
    static ref ANY_REGEX: Regex =
        Regex::new(r"^(>|#|uhh\s|le\s)(info$|meme(info|\s.*)?)$").unwrap();
    static ref GET_REGEX: Regex =
        Regex::new(r"^(>|#|uhh\s|le\s)meme$").unwrap();
    static ref ADD_REGEX: Regex =
        Regex::new(r"^(>|#|uhh\s|le\s)meme\s(.*)$").unwrap();
    static ref INFO_REGEX: Regex =
        Regex::new(r"^(>|#|uhh\s|le\s)(meme)?info$").unwrap();
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
struct Meme {
    date: DateTime<UTC>,
    author: String,
    content: String
}

#[derive(Debug)]
pub struct MemePlugin {
    file: File,
    memes: Vec<Meme>,
    last_used_map: HashMap<String, DateTime<UTC>>,
    banlist_map: HashMap<String, DateTime<UTC>>,
    last_meme: Option<Meme>,
    cooldown: Duration,
    ban_duration: Duration
}

impl Plugin for MemePlugin {
    fn new() -> Box<Plugin> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(FILE_PATH)
            .expect("Failed to open file");

        let mut rdr = csv::Reader::from_file(FILE_PATH)
            .expect("Failed to read file")
            .has_headers(false);

        let memes = rdr.decode()
            .collect::<csv::Result<Vec<Meme>>>()
            .expect("Failed to decode reader");

        Box::new(MemePlugin {
            file,
            memes,
            last_used_map: HashMap::new(),
            banlist_map: HashMap::new(),
            last_meme: None,
            cooldown: Duration::seconds(60),
            ban_duration: Duration::minutes(10)
        })
    }

    fn is_match(&self, msg: &Message) -> bool {
        ANY_REGEX.is_match(&msg.payload)
    }

    fn handle(&mut self, msg: &Message) {
        let msg_1 = msg.to_owned();
        let content = msg_1.payload;

        // Get a random meme
        if GET_REGEX.is_match(&content) {
            if self.is_banned(msg) { return };

            let meme = match rand::thread_rng().choose(&self.memes) {
                Some(m) => m,
                None => {
                    return msg.reply("Could not get a meme ugh =.= smh @ shy imouto");
                }
            };

            self.last_meme = Some(meme.to_owned());
            return msg.reply(meme.clone().content);
        }

        // Add a meme
        else if ADD_REGEX.is_match(&content) {
            if self.is_banned(msg) { return };

            let caps = ADD_REGEX.captures(&content).unwrap();

            let meme = Meme {
                date: UTC::now(),
                author: msg_1.user.name,
                content: caps.get(2).unwrap().as_str().to_owned()
            };

            if self.exists(&meme.content) {
                return msg.reply(meme.content + " is already a meme you dip");
            };

            let mut wtr = csv::Writer::from_memory();
            wtr.encode(meme.clone()).unwrap();
            self.file.write_all(wtr.as_bytes()).unwrap();

            self.memes.push(meme.clone());
            return msg.reply(meme.content + "is now a meme");
        }

        // Get meme info
        else if INFO_REGEX.is_match(&content) {
            match self.last_meme.clone() {
                None => return,
                Some(m) => {
                    return msg.reply(format!("This meme was added by {} at {}",
                                      m.author, m.date.to_rfc2822()));
                }
            }
        }
    }
}

impl MemePlugin {
    fn is_banned(&mut self, msg: &Message) -> bool {
        let now = UTC::now();
        let user = helpers::sanitize(&msg.user.name);

        // Unban the user if their ban is over
        if self.banlist_map.contains_key(&user) {
            let ban_time = *self.banlist_map.get(&user).unwrap();
            if now.signed_duration_since(ban_time) >= self.ban_duration {
                self.banlist_map.remove(&user);
            } else {
                return true
            }
        }

        // Ban the user if they used twice within one cooldown duration
        if self.last_used_map.contains_key(&user) {
            let last_used = *self.last_used_map.get(&user).unwrap();
            if now.signed_duration_since(last_used) < self.cooldown {
                self.banlist_map.insert(user, now);
                msg.send(self.ban_message(&msg.user.name));
                return true
            }
        }

        self.last_used_map.insert(user, now);

        false
    }

    fn ban_message(&self, user: &str) -> String {
        let cooldown_secs = self.cooldown.num_seconds();
        let ban_mins = self.ban_duration.num_minutes();
        let notice = "**Slow down with those memes...kid**";
        format!("{} ({} is banned from meme for {} minutes. \
            Currently allowed 1 meme per {} seconds)",
            notice, user, ban_mins, cooldown_secs)
    }

    fn exists(&self, meme: &str) -> bool {
        let mut regex_builder = RegexBuilder::new(meme);
        let regex = &regex_builder
                    .case_insensitive(true)
                    .ignore_whitespace(true)
                    .build()
                    .unwrap();

        // Create a threadpool holding 4 threads. Memes are in an unordered
        // vector so we must use brute force to check for matches.
        let mut pool = Pool::new(4);

        // TODO Don't think atomic bool is necessary, but investigate this.
        let mut exists = false;

        pool.scoped(|scoped| {
            for m in &self.memes {
                scoped.execute(move || {
                    if regex.is_match(&m.content) {
                        exists = true
                    };
                });

                if exists { break };
            }
        });

        exists
    }
}
