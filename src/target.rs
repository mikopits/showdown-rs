use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use helpers::sanitize;

/// A `Target` for the bot to reply to.
pub trait Target: Sync + Clone {
    fn new(name: &str) -> Self;
    fn send(&self, bot: &Arc<Mutex<::Bot>>, text: &str);
}

/// A `Room` implements `Target`. If the bot replies to a chat message from
/// within a room, then it will reply within the same room.
///
/// A `Room` is uniquely identified by its `name`.
#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    users: Arc<Mutex<BTreeSet<String>>>,
}

impl Target for Room {
    /// Creates a new `Room`
    fn new(name: &str) -> Self {
        Room {
            name: sanitize(name),
            users: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    fn send(&self, bot: &Arc<Mutex<::Bot>>, text: &str) {
        let to_send = format!("{}|{}", self.name, text);
        if to_send.len() > 300 {
            bot.lock().unwrap().send(&to_send[..299])
        } else {
            bot.lock().unwrap().send(to_send)
        }
    }
}

impl Room {
    fn insert_user(&mut self, name: &str) -> bool {
        self.users.lock().unwrap()
            .insert(sanitize(name))
    }

    fn remove_user(&mut self, name: &str) -> bool {
        self.users.lock().unwrap()
            .remove(&sanitize(name))
    }

    fn contains_user(&self, name: &str) -> bool {
        self.users.lock().unwrap()
            .contains(&sanitize(name))
    }
}

/// A `User`. Contains their name and known authorization levels.
///
/// A `User` is uniquely identified by `sanitize`ing its `name`.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    auths: Arc<Mutex<HashMap<String, String>>>,
}

impl Target for User {
    /// Creates a new `User`
    fn new(name: &str) -> Self {
        User {
            id: sanitize(name),
            name: String::from(name),
            auths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sends a private message to a `User`.
    fn send(&self, bot: &Arc<Mutex<::Bot>>, text: &str) {
        let to_send = format!("|/w {},{}", self.name, text);
        if to_send.len() > 300 {
            bot.lock().unwrap().send(&to_send[..299])
        } else {
            bot.lock().unwrap().send(to_send)
        }
    }
}

impl User {
    /// Adds an authorization level in a room.
    fn add_auth(&mut self, auth: &str, room: &str) {
        self.auths.lock().unwrap()
            .entry(sanitize(room))
            .or_insert(String::from(auth));
    }

    fn has_auth(&self, auth: &str, room: &str) -> bool {
        match self.auths.lock().unwrap().get(room) {
            Some(v) => {
                if v == auth {
                    true
                } else {
                    false
                }
            },
            None => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheMap<T: Target> {
    pub map: Arc<Mutex<HashMap<String, T>>>
}

impl<T: Target> CacheMap<T> {
    pub fn new() -> Self {
        CacheMap { map: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn insert(&mut self, name: &str) {
        self.map.lock().unwrap()
            .entry(sanitize(name))
            .or_insert(Target::new(name));
    }

    pub fn remove(&mut self, name: &str) {
        self.map.lock().unwrap().remove(&sanitize(name)).unwrap();
    }

    pub fn contains(&self, name: &str) -> bool {
        self.map.lock().unwrap()
            .contains_key(&sanitize(name))
    }
}

impl CacheMap<Room> {
    pub fn insert_user_in_room(&mut self, u: &str, r: &str) -> bool {
        let mut map = self.map.lock().unwrap();
        map.entry(sanitize(r))
            .or_insert(Target::new(r))
            .insert_user(u)
    }

    pub fn remove_user_from_room(&mut self, u: &str, r: &str) -> bool {
        let mut map = self.map.lock().unwrap();
        map.entry(sanitize(r))
            .or_insert(Target::new(r))
            .remove_user(u)
    }

    pub fn contains_user_in_room(&self, u: &str, r: &str) -> bool {
        match self.map.lock().unwrap().get(r) {
            Some(room) => room.contains_user(u),
            None => false
        }
    }
}

impl CacheMap<User> {
    pub fn add_auth_to_user_in_room(&mut self, a: &str, u: &str, r: &str) {
        let mut map = self.map.lock().unwrap();
        map.entry(sanitize(u))
            .or_insert(Target::new(u))
            .add_auth(a, r);
    }
}

#[cfg(test)]
mod tests {
    use ::bot::Bot;

    static TEST_PATH: &'static str = "examples/example_config.toml";

    #[test]
    fn add_user_test() {
        let mut b = Bot::new(TEST_PATH).unwrap();
        assert!(!b.room_map.contains("testroom"));
        assert!(!b.room_map.contains_user_in_room("testuser", "testroom"));
        b.room_map.insert_user_in_room("testuser", "testroom");
        assert!(b.room_map.contains("testroom"));
        assert!(b.room_map.contains_user_in_room("testuser", "testroom"));
    }

    #[test]
    fn remove_user_test() {
        let mut b = Bot::new(TEST_PATH).unwrap();
        b.room_map.insert_user_in_room("testuser", "testroom");
        assert!(b.room_map.contains_user_in_room("testuser", "testroom"));
        b.room_map.remove_user_from_room("testuser", "testroom");
        assert!(!b.room_map.contains_user_in_room("testuser", "testroom"));
    }
}
