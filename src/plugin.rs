use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam;
use regex::{Regex, escape};
use time::{Duration, Tm, empty_tm};

use bot::Bot;
use message::Message;

pub trait CustomPlugin {
    fn handle();
}

#[derive(Debug)]
pub struct Plugin<'a> {
    bot: &'a Arc<Mutex<Bot>>,
    name: String,
    command: String,
    pattern: Regex,
    num_args: i8,
    cooldown: Duration,
    last_used: Tm,
}

impl<'a> CustomPlugin for Plugin<'a> {
    fn handle() {}
}

impl<'a> Plugin<'a> {
    pub fn new(bot: &'a Arc<Mutex<Bot>>, name: &'a str, command: &'a str,
               num_args: i8, cooldown: Option<Duration>) -> Self {
        Plugin {
            bot: bot,
            name: String::from(name),
            command: escape(name),
            pattern: generate_pattern(bot, command, num_args),
            num_args: num_args,
            cooldown: match cooldown {
                Some(d) => d,
                None => Duration::zero(),
            },
            last_used: empty_tm()
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.pattern.is_match(text)
    }

    pub fn update_last_used(&mut self, tm: Tm) {
        self.last_used = tm;
    }

    pub fn execute(&mut self, msg: &Message) {
        thread::spawn(move || {
            let args = arguments(&msg.payload, self.pattern);
            info!("Starting chat event handler thread for plugin {} with args {:?}",
                  self.name, args);
            if msg.received - self.last_used > self.cooldown {
                self.update_last_used(msg.received);
                self.handle();
            }
        }).join();
    }

    /*pub fn listen(&mut self) {
        info!("Started listening on plugin: {}", self.name);

        crossbeam::scope(|scope| {
            let self_1 = self.clone();
            loop {
                let message = self_1.bot.lock().unwrap().recv_message();
                if self_1.is_match(&message.payload) {
                    let args = arguments(&message.payload, self_1.pattern);
                    info!("Starting chat event handler thread for plugin {} with args {:?}",
                          self_1.name, args);
                    if &message.received - self.last_used > self_1.cooldown {
                        self_1.updated_last_used(message.received);
                        scope.spawn(move || {
                            debug!("Spawned scoped plugin thread");
                        });
                    }
                }
            }
        });

        info!("Stopped listening on plugin: {}", self.name);
    }*/
}

fn generate_pattern(bot: &Arc<Mutex<Bot>>, command: &str, num_args: i8)
    -> Regex {
        let config = bot.lock().unwrap().config.clone();
        let flags = match config.case_sensitive {
            true  => "",
            false => "(?i)",
        };
        let pre = &config.plugin_prefixes.iter()
            .map(|p| escape(&p)).collect::<Vec<String>>()
            .join("|");

        // 0 argument case
        if num_args == 0 {
            return Regex::new(&["^(", flags, "(", pre, ")",
                              command, ")$"].concat()).unwrap()
        }

        let mut args: Vec<&str> = Vec::new();

        // 1 argument case
        if num_args == 1 {
            args.push("\\s+(.+)");
        } else {
            args.push("\\s+([^,]+)");
        }

        // 2 or more argument case
        if num_args > 1 {
            for i in 1..num_args {
                if i == num_args - 2 {
                    args.push(",\\s+(.+)");
                } else {
                    args.push(",\\s+([^,]+)");
                }
            }
        }

        Regex::new(&["^(", flags, "(", pre, ")",
                   command, &args.concat(), "$)"].concat()).unwrap()
}

pub fn arguments(text: &str, re: Regex) -> Vec<String> {
    let mut captures: Vec<String> = Vec::new();
    let caps = re.captures(text).unwrap();
    for i in 2..re.captures_len() {
        captures.push(String::from(caps.get(i).unwrap().as_str()));
    }
    captures
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use regex::Regex;
    use ::bot::Bot;
    use ::plugin::Plugin;

    static TEST_PATH_S: &'static str = "src/test/test_sensitive.toml";
    static TEST_PATH_I: &'static str = "src/test/test_insensitive.toml";

    #[test]
    fn no_args_test() {
        let b = &Arc::new(Mutex::new(Bot::new(TEST_PATH_S)));
        let p = Plugin::new(b, "test", "test", 0, None);
        assert!(p.is_match(".test"));
        assert!(p.is_match("#test"));
        assert!(!p.is_match(".test test"));
        assert!(!p.is_match("atest"));
        assert!(!p.is_match(".TeSt"));
        let bi = &Arc::new(Mutex::new(Bot::new(TEST_PATH_I)));
        let p = Plugin::new(bi, "test", "test", 0, None);
        assert!(p.is_match(".TeSt"));
    }

    #[test]
    fn one_arg_test() {
        let b = &Arc::new(Mutex::new(Bot::new(TEST_PATH_S)));
        let p = Plugin::new(b, "test", "test", 1, None);
        assert!(p.is_match(".test test,ing,,,"));
        assert!(!p.is_match(".test"));
    }

    #[test]
    fn two_args_test() {
        let b = &Arc::new(Mutex::new(Bot::new(TEST_PATH_S)));
        let p = Plugin::new(b, "test", "test", 2, None);
        assert!(p.is_match(".test test, testing"));
        assert!(p.is_match(".test test,   test"));
        assert!(!p.is_match(".test test"));
    }

    #[test]
    fn three_args_test() {
        let b = &Arc::new(Mutex::new(Bot::new(TEST_PATH_S)));
        let p = Plugin::new(b, "test", "test", 3, None);
        assert!(p.is_match(".test test, test, test"));
        assert!(!p.is_match(".test test, test"));
    }
}
