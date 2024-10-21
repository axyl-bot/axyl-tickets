use std::env;

pub struct Config {
    pub token: String,
    pub category_id: u64,
    pub log_channel_id: u64,
}

impl Config {
    pub fn get() -> Self {
        Self {
            token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
            category_id: env::var("CATEGORY_ID")
                .expect("CATEGORY_ID must be set")
                .parse()
                .expect("CATEGORY_ID must be a valid u64"),
            log_channel_id: env::var("LOG_CHANNEL_ID")
                .expect("LOG_CHANNEL_ID must be set")
                .parse()
                .expect("LOG_CHANNEL_ID must be a valid u64"),
        }
    }
}
