use std::sync::{Arc, Mutex};
use time::{Tm, now};

use target::{Target, User, Room};

/// A `Message` is a message from the server, parsed to make sense of
/// Pokemon Showdown's custom protocol.
#[derive(Clone, Debug)]
pub struct Message<'a> {
    bot: &'a Arc<Mutex<::Bot>>,
    pub received: Tm,
    pub timestamp: u32,
    pub command: String,
    pub params: Vec<String>,
    pub private: bool,
    pub room: Room,
    pub user: User,
    pub auth: String,
    pub payload: String,
}

impl<'a> Message<'a> {
    /// Creates a new `Message` by serializing the message in text form.
    pub fn from_string(text: String, bot: &'a Arc<Mutex<::Bot>>) -> Self {
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
            bot,
            received,
            timestamp,
            command,
            params,
            private,
            room: Target::new(&room),
            user: Target::new(&user),
            auth,
            payload,
        }
    }

    /// Handles server messages.
    pub fn handle(&self, bot: &'a Arc<Mutex<::Bot>>) -> ::Result<()> {
        match &*self.command {
            // |battle|ROOMID|USER1|USER2 or |b|ROOMID|USER1|USER2
            "b" | "battle" => Ok(()),

            // |challstr|CHALLSTR
            "challstr" => {
                info!("Attempting to log in...");
                bot.lock().unwrap().login(
                    &format!("{}|{}", &self.params[0], &self.params[1]))?;
                Ok(())
            },

            // |c:|TIMESTAMP|USER|MESSAGE
            // Chat events are handled in the receive loop.
            "c:" => Ok(()),

            // |formats|FORMATSLIST
            "formats" => Ok(()),

            // |html|HTML
            "html" => Ok(()),

            // |init|ROOMTYPE
            "init" => Ok(()),

            // |join|USER or |j|USER
            "j" | "join" => {
                bot.lock().unwrap().room_map
                    .insert_user_in_room(&self.user.name, &self.room.name);
                bot.lock().unwrap().user_map
                    .add_auth_to_user_in_room(
                        &self.auth, &self.user.name, &self.room.name);
                Ok(())
            },

            // |leave|USER or |l|USER
            "l" | "leave" => {
                bot.lock().unwrap().room_map
                    .remove_user_from_room(&self.user.name, &self.room.name);
                Ok(())
            },

            // ||MESSAGE or MESSAGE
            "" => Ok(()),

            // |nametaken|USERNAME|MESSAGE
            "nametaken" => Ok(()),

            // |name|USER|OLDID or |n|USER|OLDID
            "n" | "name" => {
                bot.lock().unwrap().room_map
                    .remove_user_from_room(&self.params[0], &self.room.name);
                bot.lock().unwrap().room_map
                    .insert_user_in_room(&self.user.name, &self.room.name);
                bot.lock().unwrap().user_map
                    .add_auth_to_user_in_room(
                        &self.auth, &self.user.name, &self.room.name);
                Ok(())
            },

            // |popup|MESSAGE
            "popup" => Ok(()),

            // |pm|SENDER|RECEIVER|MESSAGE
            "pm" => Ok(()),

            // |queryresponse|QUERYTYPE|JSON
            "queryresponse" => Ok(()),

            // |tie
            "tie" => Ok(()),

            // |:|TIMESTAMP
            ":" => {
                bot.lock().unwrap().set_login_time(self.timestamp);
                Ok(())
            },

            // |uhtml|NAME|HTML
            "uhtml" => Ok(()),

            // |uhtmlchange|NAME|HTML
            "uhtmlchange" => Ok(()),

            // |updatechallenges|JSON
            "updatechallenges" => Ok(()),

            // |updatesearch|JSON
            "updatesearch" => Ok(()),

            // |updateuser|USERNAME|NAMED|AVATAR
            "updateuser" => match &*self.params[1] {
                "0" => {
                    let avatar = bot.lock().unwrap()
                        .config.avatar;
                    if avatar > 0 && avatar <= 294 {
                        bot.lock().unwrap()
                            .send(format!("|/avatar {}", avatar));
                    }
                    Ok(())
                },
                "1" => {
                    let config = bot.lock().unwrap().config.clone();
                    for r in &config.rooms {
                        bot.lock().unwrap().join_room(&r);
                    }
                    // TODO: start timed plugins
                    Ok(())
                },
                _ => {
                    unreachable!();
                }
            },

            // |usercount|USERCOUNT
            "usercount" => Ok(()),

            // |users|USERLIST
            "users" => {
                for user in self.params[0].split(",").skip(1) {
                    let auth: String = user.chars().take(1).collect();
                    let user: String = user.chars().skip(1).collect();
                    bot.lock().unwrap().room_map
                        .insert_user_in_room(&user, &self.room.name);
                    bot.lock().unwrap().user_map
                        .add_auth_to_user_in_room(&auth, &user, &self.room.name);
                }
                Ok(())
            },

            // |win|USER
            "win" => Ok(()),

            // Ignore commands we have no plan for
            _ => Ok(())
        }
    }

    pub fn reply<S: Into<String>>(&self, text: S) {
        let msg = &format!("({}) {}", self.user.name, text.into());
        match self.private {
            false => { self.room.send(self.bot, msg); },
            true  => { self.user.send(self.bot, msg); }
        }
    }

    pub fn send<S: Into<String>>(&self, text: S) {
        match self.private {
            false => { self.room.send(self.bot, &text.into()); },
            true  => { self.user.send(self.bot, &text.into()); }
        }
    }

    pub fn prefix_string(&self) -> String {
        self.bot.lock().unwrap().config.prefix_string()
    }
}
