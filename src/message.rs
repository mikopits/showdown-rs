use time::{Tm, now};
use std::sync::{Arc, Mutex};

use bot::Bot;
use target::{Target, User, Room};

/// A `Message` is a message from the server, parsed to make sense of
/// Pokemon Showdown's custom protocol.
#[derive(Debug)]
pub struct Message<'a> {
    bot: &'a Arc<Mutex<Bot>>,
    pub received: Tm,
    pub timestamp: u32,
    pub command: String,
    pub params: Vec<String>,
    private: bool,
    pub room: Room,
    pub user: User,
    pub auth: String,
    pub payload: String,
}

impl<'a> Message<'a> {
    /// Creates a new `Message` by serializing the message in text form.
    pub fn from_string(text: String, bot: &'a Arc<Mutex<Bot>>) -> Self {
        let received = now();

        let nl_delim: Vec<&str> = text.split("\n").collect();
        let vb_delim: Vec<&str> = text.split("|").collect();

        // The command is always after the first vertical bar
        let mut command = String::new();
        if vb_delim.len() > 1 {
            command = String::from(vb_delim[1]).to_lowercase();
        }

        // Parse the parameters following a command
        let mut params: Vec<String> = Vec::new();
        if !command.is_empty() && vb_delim.len() > 2 {
            for s in &vb_delim[2..] {
                params.push(String::from(&**s));
            }
        }

        // Parse the UNIX timestamp of a chat event
        let mut timestamp: u32 = 0;
        if command.contains(":") {
            timestamp = match params[0].parse::<u32>() {
                Ok(i) => i,
                Err(e) => {
                    error!("Message From Text: {:?}", e);
                    0
                }
            };
        }

        // If the message starts with a ">" then it comes from a room
        let mut room = String::new();
        if nl_delim.len() > 0 {
            if !nl_delim.first().unwrap().is_empty() {
                if nl_delim.first().unwrap().as_bytes()[0] == 62u8 {
                    room =  String::from(&nl_delim[0][1..]);
                }
            }
        }

        // Parse the user sending a command, and their auth level, and if the
        // message was private, and the payload
        let mut auth = String::new();
        let mut user = String::new();
        let mut payload = String::new();
        let mut private = false;
        match &*command {
            "c:" => {
                auth = String::from(&vb_delim[3][..0]);
                user = String::from(&vb_delim[3][1..]);
                payload = vb_delim[4..].join("|");
            },
            "pm" => {
                auth = String::from(&vb_delim[2][..0]);
                user = String::from(&vb_delim[2][1..]);
                payload = vb_delim[4..].join("|");
                private = true;
            }
            "c" | "j" | "l" | "n" => {
                auth = String::from(&vb_delim[2][..0]);
                user = String::from(&vb_delim[2][1..]);
            }
            _ => {
                payload = String::from(&**nl_delim.last().unwrap());
            }
        }

        // Update state
        // TODO: This shouldn't be here. do when necessary in j/l/n events.
        if !room.is_empty() {
            bot.lock().unwrap().room_map.insert(&room);
        }
        if !user.is_empty() {
            bot.lock().unwrap().user_map.insert(&user);
        }
        if !(user.is_empty() || room.is_empty()) {
            bot.lock().unwrap()
                .room_map.insert_user_in_room(&user, &room);
            bot.lock().unwrap()
                .user_map.add_auth_to_user_in_room(&auth, &user, &room);
        }

        Message {
            bot: bot,
            received: received,
            timestamp: timestamp,
            command: command,
            params: params,
            private: private,
            room: Target::new(&room),
            user: Target::new(&user),
            auth: auth,
            payload: payload,
        }
    }

    /// Handles server messages.
    pub fn handle(&self, bot: &Arc<Mutex<Bot>>) {
        match &*self.command {
            // |battle|ROOMID|USER1|USER2 or |b|ROOMID|USER1|USER2
            "b" | "battle" => return,

            // |challstr|CHALLSTR
            "challstr" => {
                info!("Attempting to log in...");
                bot.lock().unwrap().login(
                    &format!("{}|{}", &self.params[0], &self.params[1]));
            },

            // |c:|TIMESTAMP|USER|MESSAGE
            "c:" => {
                if bot.lock().unwrap().login_time == 0 { return; }
                // TODO: Handle plugins
                /*if !self.payload.is_empty() &&
                    self.timestamp >= bot.lock().unwrap().login_time {
                        bot.lock().unwrap().send_message(self);
                    }*/
            },

            // |formats|FORMATSLIST
            "formats" => return,

            // |html|HTML
            "html" => return,

            // |init|ROOMTYPE
            "init" => return,

            // |join|USER or |j|USER
            "j" | "join" => {
                bot.lock().unwrap().room_map
                    .insert_user_in_room(&self.user.name, &self.room.name);
                bot.lock().unwrap().user_map
                    .add_auth_to_user_in_room(
                        &self.auth, &self.user.name, &self.room.name);
            },

            // |leave|USER or |l|USER
            "l" | "leave" => {
                bot.lock().unwrap().room_map
                    .remove_user_from_room(&self.user.name, &self.room.name);
            },

            // ||MESSAGE or MESSAGE
            "" => return,

            // |nametaken|USERNAME|MESSAGE
            "nametaken" => return,

            // |name|USER|OLDID or |n|USER|OLDID
            "n" | "name" => {
                bot.lock().unwrap().room_map
                    .remove_user_from_room(&self.params[0], &self.room.name);
                bot.lock().unwrap().room_map
                    .insert_user_in_room(&self.user.name, &self.room.name);
                bot.lock().unwrap().user_map
                    .add_auth_to_user_in_room(
                        &self.auth, &self.user.name, &self.room.name);
            },

            // |popup|MESSAGE
            "popup" => return,

            // |pm|SENDER|RECEIVER|MESSAGE
            "pm" => return,

            // |queryresponse|QUERYTYPE|JSON
            "queryresponse" => return,

            // |tie
            "tie" => return,

            // |:|TIMESTAMP
            ":" => {
                bot.lock().unwrap().set_login_time(self.timestamp);
            },

            // |uhtml|NAME|HTML
            "uhtml" => return,

            // |uhtmlchange|NAME|HTML
            "uhtmlchange" => return,

            // |updatechallenges|JSON
            "updatechallenges" => return,

            // |updatesearch|JSON
            "updatesearch" => return,

            // |updateuser|USERNAME|NAMED|AVATAR
            "updateuser" => match &*self.params[1] {
                "0" => {
                    let avatar = bot.lock().unwrap()
                        .config.avatar;
                    if avatar > 0 && avatar <= 294 {
                        bot.lock().unwrap()
                            .send(format!("|/avatar {}", avatar));
                    }
                },
                "1" => {
                    let config = bot.lock().unwrap().config.clone();
                    for r in &config.rooms {
                        bot.lock().unwrap().join_room(&r);
                    }
                    // TODO: start timed plugins
                }
                _ => {
                    unreachable!();
                }
            },

            // |usercount|USERCOUNT
            "usercount" => return,

            // |users|USERLIST
            "users" => {
                for user in self.params[0].split(",").skip(1) {
                    let auth_bytes: Vec<u8> = user.bytes().clone()
                        .take(1).collect();
                    let user_bytes: Vec<u8> = user.bytes().clone()
                        .skip(1).collect();
                    let auth = &String::from_utf8(auth_bytes).unwrap();
                    let name = &String::from_utf8(user_bytes).unwrap();
                    bot.lock().unwrap().room_map
                        .insert_user_in_room(name, &self.room.name);
                    bot.lock().unwrap().user_map
                        .add_auth_to_user_in_room(auth, name, &self.room.name);
                }
            },

            // |win|USER
            "win" => return,

            // Ignore commands we have no plan for
            _ => return,
        }
    }

    #[allow(dead_code)]
    pub fn reply(&self, text: &str) {
        match self.private {
            false => { self.room.send(self.bot, text); },
            true  => { self.user.send(self.bot, text); },
        }
    }
}
