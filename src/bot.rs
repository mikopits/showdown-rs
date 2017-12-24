use std::thread;
use std::time::Duration;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::io::{Read, stdin};
use std::path::Path;
use std::sync::{Arc, Mutex, mpsc};

use serde_json::Value;
use websocket::{ClientBuilder, Message};
use websocket::url::Url;
use websocket::message::Type;

use target::{CacheMap, Room, User};

/// A `Bot` contains all the bot functionality. It is recommended to only use
/// one bot even on multiple rooms so that all your messages are throttled.
#[derive(Clone, Debug)]
pub struct Bot {
    pub login_time: u32,
    pub config: ::Config,
    rooms_in: BTreeSet<String>,
    pub user_map: CacheMap<User>,
    pub room_map: CacheMap<Room>,
    tx: Arc<Mutex<mpsc::Sender<Message<'static>>>>,
    rx: Arc<Mutex<mpsc::Receiver<Message<'static>>>>,
    plugins: Arc<Mutex<Vec<Arc<Mutex<Box<::Plugin>>>>>>
}

impl Bot {
    /// Creates a new `Bot` from a config path. If you are calling cargo run
    /// from your cargo root, then the config_path root will be the cargo root.
    /// Returns an Error if the config file is not found.
    pub fn new<P>(config_path: P) -> ::Result<Bot>
        where P: AsRef<Path>,
    {
        let (tx, rx) = mpsc::channel();
        Ok(Bot {
            login_time: 0,
            config: ::Config::new(config_path)?,
            rooms_in: BTreeSet::new(),
            user_map: CacheMap::new(),
            room_map: CacheMap::new(),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            plugins: Arc::new(Mutex::new(Vec::new()))
        })
    }

    /// Initialize the websocket connection to the server. The entrypoint
    /// method to all Bot functionality and runs the main loop. Returns an
    /// error if the bot cannot connect to the server.
    pub fn connect(self) -> ::Result<()> {
        let url = Url::parse(
            &format!("ws://{}:{}/showdown/websocket",
            &self.config.host,
            &self.config.port))?;

        info!("Connecting to {}", url);
        let client = ClientBuilder::from_url(&url).connect_insecure()?;

        info!("Successfully connected");
        let (mut receiver, mut sender) = client.split()?;

        let self_1 = Arc::new(Mutex::new(self.clone()));
        let self_2 = self_1.clone();
        let tx = self_1.lock().unwrap().to_owned().tx;
        let rx = self_1.lock().unwrap().to_owned().rx;
        let tx_1 = tx.clone();
        let plugins = self.plugins.lock().unwrap().clone();

        debug!("Spawning send loop thread");
        let send_loop = thread::spawn(move || {
            loop {
                let message: Message = match rx.lock().unwrap().recv() {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = sender.send_message(&Message::close());
                        return Err(e)
                    }
                };

                // If it's a close message, send it and return
                match message.opcode {
                    Type::Close => {
                        let _ = sender.send_message(&message);
                        return Ok(());
                    },
                    _ => ()
                }

                // Stringify the payload
                let text = match String::from_utf8(
                    message.clone().payload.into_owned()) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        return Ok(());
                    }
                };

                info!("\x1b[33m↵\x1b[0m{}", text);

                // Send the message
                match sender.send_message(&message) {
                    Ok(()) => {
                        thread::sleep(
                            Duration::from_millis(
                                self_1.lock().unwrap().config.throttle_ms));
                    },
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        let _ = sender.send_message(&Message::close());
                        return Ok(());
                    }
                }
            }
        });

        debug!("Spawning receive loop thread");
        let recv_loop = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: Message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Receive Loop: {:?}", e);
                        let _ = tx.lock().unwrap().send(Message::close());
                        return;
                    }
                };

                match message.opcode {
                    // Send closure when closure is received
                    Type::Close => {
                        let _ = tx.lock().unwrap().send(Message::close());
                        return;
                    },

                    // Pong when pinged
                    Type::Ping => match tx.lock().unwrap()
                        .send(Message::pong(message.payload)) {
                        Ok(()) => (),
                        Err(e) => {
                            error!("Receive Loop: {:?}", e);
                            return;
                        }
                    },

                    // Handle a normal server message
                    _ => {
                        let payload = match String::from_utf8(
                            message.payload.into_owned()) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Receive Loop: {:?}", e);
                                return;
                            }
                        };

                        let mut room = "";
                        let mut messages: Vec<&str> = payload.split("\n").collect();

                        let first: Vec<u8> = messages[0].bytes().take(1).collect();

                        if first[0] == 62u8 {
                            room = messages[0];
                            messages = messages[1..].to_vec();
                        }

                        for message in messages {
                            info!("\x1b[32m↳\x1b[0m{}", room.to_owned() + message);

                            let m = ::Message::from_string(String::from(
                                    format!("{}\n{}", room, message)), &self_2);

                            match m.handle(&self_2) {
                                Err(e) => {
                                    error!("Failed to handle message: {:?}", e);
                                    return;
                                },
                                _ => (),
                            }

                            if !m.payload.is_empty() && m.timestamp >=
                                self_2.lock().unwrap().login_time {
                                for p in plugins.iter()
                                    .filter(|&p| p.lock().unwrap().is_match(&m)) {
                                    //debug!("[plugin] Spawning thread for plugin");
                                    //thread::spawn(move || {
                                        p.lock().unwrap().handle(&m);
                                    //});
                                }
                            }
                        }
                    },
                }
            }
        });

        loop {
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            let trimmed = input.trim();
            let message = match trimmed {
                "/close" => {
                    let _ = tx_1.lock().unwrap().send(Message::close());
                    break;
                }
                "/ping" => Message::ping(b"PING".to_vec()),
                _ => Message::text(trimmed.to_string()),
            };

            match tx_1.lock().unwrap().send(message) {
                Ok(()) => (),
                Err(e) => {
                    error!("Send Loop: {:?}", e);
                    break;
                }
            }
        }

        info!("Waiting for child threads to exit...");

        let _ = send_loop.join();
        let _ = recv_loop.join();

        info!("Exited");
        Ok(())
    }

    pub fn register(&self, plugin: Box<::Plugin>) {
        self.plugins.lock().unwrap().push(Arc::new(Mutex::new(plugin)));
    }

    /// Send a `String` to the websocket. For convenience, allow any Type that
    /// implements `Into<String>`.
    pub fn send<S: Into<String>>(&self, text: S) {
        match self.tx.lock().unwrap().send(Message::text(text.into())) {
            Err(e) => {
                error!("Failed to send to websocket: {:?}", e);
                return;
            }
            _ => return,
        }
    }

    /// Join a room and update the state given the room name.
    pub fn join_room(&mut self, name: &str) {
        self.room_map.insert(name);
        self.rooms_in.insert(String::from(name));
        self.send(format!("|/join {}", name));
    }

    /// Leave a room and update the state given the room name.
    pub fn leave_room(&mut self, name: &str) {
        self.room_map.remove(name);
        self.rooms_in.remove(name);
        self.send(format!("|/leave {}", name));
    }

    /// Set the login time.
    pub fn set_login_time(&mut self, timestamp: u32) {
        self.login_time = timestamp;
    }

    pub fn login(&self, challstr: &str) -> ::Result<()> {
        let (user, pass) = {
            let u = env::var("BOT_USERNAME").unwrap();
            let p = env::var("BOT_PASSWORD").unwrap();

            (u, p)
        };
        let client = ::reqwest::Client::new().unwrap();
        let sanitized_user = &::helpers::sanitize(&user);

        let mut params = HashMap::new();
        if pass.is_empty() {
            params.insert("act", "getassertion");
            params.insert("userid", sanitized_user);
        } else {
            params.insert("act", "login");
            params.insert("name", &user);
            params.insert("pass", &pass);
        }
        params.insert("challstr", challstr);

        let mut res = client
            .post("https://play.pokemonshowdown.com/action.php")
            .form(&params)
            .send()?;

        let mut buf = String::new();
        res.read_to_string(&mut buf)?;
        let data_str = &buf[1..];

        let v: Value = ::serde_json::from_str(&data_str)?;
        let assertion = v["assertion"].as_str().unwrap();

        self.send(format!("|/trn {},0,{}", user, assertion));
        Ok(())
    }
}
