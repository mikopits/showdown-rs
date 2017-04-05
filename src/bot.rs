use std::{thread, time};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use reqwest::{Client as HTTPClient};
use rustc_serialize::json::Json;
use websocket::Client as WSClient;
use websocket::{Message as WSMessage, Sender, Receiver};
use websocket::client::request::Url;
use websocket::message::Type;

use config::Config;
use message::Message;
use target::{CacheMap, Room, User};

/// A `Bot` client that connects to the pokemon showdown server via websocket.
#[derive(Clone, Debug)]
pub struct Bot {
    host: String,
    port: String,
    pub login_time: u32,
    pub config: Arc<Config>,
    rooms_in: Arc<Mutex<BTreeSet<String>>>,
    pub user_map: CacheMap<User>,
    pub room_map: CacheMap<Room>,
    tx: Arc<Mutex<mpsc::Sender<WSMessage<'static>>>>,
    rx: Arc<Mutex<mpsc::Receiver<WSMessage<'static>>>>,
    /*ptx: Arc<Mutex<mpsc::Sender<&'static Message<'static>>>>,
    prx: Arc<Mutex<mpsc::Receiver<&'static Message<'static>>>>,*/
}

impl Bot {
    /// Construct a new `Bot`, taking the path to the configuration toml file
    /// as an argument.
    ///
    /// As of now, all fields of the toml file are necessary for the bot to
    /// compile.
    pub fn new<S: Into<String>>(config_path: S) -> Self {
        let (tx, rx) = mpsc::channel();
        //let (ptx, prx) = mpsc::channel();
        let config = Config::new(config_path.into());
        Bot {
            host: config.clone().host,
            port: config.clone().port,
            login_time: 0,
            config: Arc::new(config),
            rooms_in: Arc::new(Mutex::new(BTreeSet::new())),
            user_map: CacheMap::new(),
            room_map: CacheMap::new(),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            /*ptx: Arc::new(Mutex::new(ptx)),
            prx: Arc::new(Mutex::new(prx)),*/
        }
    }

    /// Initialize the websocket connection to the server. The entrypoint
    /// method to all Bot functionality and runs the main loop.
    pub fn connect(&mut self) {
        let url = match Url::parse(
            format!("ws://{}:{}/showdown/websocket",
                    &self.host, &self.port)
            .as_str()) {
            Ok(u) => u,
            Err(e) => {
                error!("Could not parse url: {:?}", e);
                return;
            }
        };

        info!("Connecting to {}", url);
        let request = match WSClient::connect(url) {
            Ok(r) => r,
            Err(e) => {
                error!("Could not connect to client: {:?}", e);
                return;
            }
        };

        let response = match request.send() {
            Ok(r) => r,
            Err(e) => {
                error!("Client request failed: {:?}", e);
                return;
            }
        };

        info!("Validating response...");
        response.validate().unwrap();

        info!("Successfully connected");
        let (mut sender, mut receiver) = response.begin().split();
        let self_1 = self.clone();
        let self_2 = self.clone();
        let self_3 = Arc::new(Mutex::new(self.clone()));
        let (tx, rx) = (self_1.tx, self_1.rx);
        let tx_1 = tx.clone();

        let send_loop = thread::spawn(move || {
            loop {
                let message: WSMessage = match rx.lock().unwrap().recv() {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        let _ = sender.send_message(&WSMessage::close());
                        return;
                    }
                };

                // If it's a close message, send it and return
                match message.opcode {
                    Type::Close => {
                        let _ = sender.send_message(&message);
                        return;
                    },
                    _ => (),
                }

                // Stringify the payload
                let text = match String::from_utf8(
                    message.clone().payload.into_owned()
                    ) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        return;
                    }
                };

                println!("\x1b[31m↵\x1b[0m{}", text);

                // Send the message
                match sender.send_message(&message) {
                    Ok(()) => {
                        let throttle_ms = time::Duration::from_millis(
                            self_2.config.clone().messages_per_ms);
                        thread::sleep(throttle_ms);
                        ()
                    },
                    Err(e) => {
                        error!("Send Loop: {:?}", e);
                        let _ = sender.send_message(&WSMessage::close());
                        return;
                    }
                }
            }
        });

        let recv_loop = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: WSMessage = match message {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Receive Loop: {:?}", e);
                        let _ = tx.lock().unwrap().send(WSMessage::close());
                        return;
                    }
                };

                match message.opcode {
                    // Send closure when closure is received and return
                    Type::Close => {
                        let _ = tx.lock().unwrap()
                            .send(WSMessage::close());
                        return;
                    }
                    // Pong when we get pinged
                    Type::Ping => match tx.lock().unwrap()
                        .send(WSMessage::pong(message.payload)) {
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
                            println!("\x1b[32m↳\x1b[0m{}",
                                     format!("{}{}", room, message));
                            let m = Message::from_string(String::from(
                                    format!("{}\n{}", room, message)), &self_3);
                            debug!("rooms: {:?}", self_3.lock().unwrap().room_map);
                            m.handle(&self_3);
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
                    let _ = tx_1.lock().unwrap().send(WSMessage::close());
                    break;
                }
                "/ping" => WSMessage::ping(b"PING".to_vec()),
                _ => WSMessage::text(trimmed.to_string()),
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
    }

    /// Send a `String` to the websocket. For convenience, allow any Type that
    /// implements `Into<String>`.
    pub fn send<S: Into<String>>(&self, s: S) {
        match self.tx.lock().unwrap().send(WSMessage::text(s.into())) {
            Err(e) => {
                error!("Failed to send to websocket: {:?}", e);
                return;
            }
            _ => return,
        }
    }
    /*
    pub fn send_message(&self, m: &Message) {
        match self.ptx.lock().unwrap().send(m) {
            Ok(()) => return,
            Err(e) => {
                error!("Failed to send message to plugins: {:?}", e);
                return;
            }
        }
    }

    pub fn recv_message(&self) -> &Message {
        match self.prx.lock().unwrap().recv() {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to receive plugin message: {:?}", e);
                panic!(e);
            }
        }
    }*/

    /// Log in using username and password in config.
    pub fn login(&self, challstr: &str) {
        let client = HTTPClient::new().unwrap();
        let config_1 = self.config.clone();
        let config_2 = self.config.clone();

        let mut params = HashMap::new();
        params.insert("act", "login");
        params.insert("name", &config_1.user);
        params.insert("pass", &config_2.pass);
        params.insert("challstr", challstr);

        let mut res = match client.post("https://play.pokemonshowdown.com/action.php")
            .form(&params)
            .send() {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to login: {:?}", e);
                    return;
                }
            };

        let mut buf = String::new();
        match res.read_to_string(&mut buf) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read to buf {:?}", e);
                return;
            }
        };

        let data_str = &buf[1..];
        let data = Json::from_str(data_str).unwrap();
        let obj = data.as_object().unwrap();
        let assertion = obj.get("assertion").unwrap().as_string().unwrap();
        self.send(format!("|/trn {},0,{}",
                          self.config.clone().user, assertion));
    }

    /// Join a room and update the state given the room name.
    pub fn join_room(&mut self, name: &str) {
        self.room_map.insert(name);
        self.rooms_in.lock().unwrap()
            .insert(String::from(name));
        self.send(format!("|/join {}", name));
    }

    /// Leave a room and update the state given the room name.
    pub fn leave_room(&mut self, name: &str) {
        self.room_map.remove(name);
        self.rooms_in.lock().unwrap()
            .remove(name);
        self.send(format!("|/leave {}", name));
    }

    /// Set the login time.
    pub fn set_login_time(&mut self, timestamp: u32) {
        self.login_time = timestamp;
    }
}
