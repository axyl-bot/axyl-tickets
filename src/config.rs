use std::env;
use std::sync::RwLock;

pub struct Config {
    pub token: String,
    pub category_id: RwLock<Option<u64>>,
    pub log_channel_id: RwLock<Option<u64>>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
            category_id: RwLock::new(None),
            log_channel_id: RwLock::new(None),
        }
    }

    pub fn set_category_id(&self, id: u64) {
        *self.category_id.write().unwrap() = Some(id);
    }

    pub fn set_log_channel_id(&self, id: u64) {
        *self.log_channel_id.write().unwrap() = Some(id);
    }
}
