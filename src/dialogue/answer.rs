use crate::db::RedisConnection;
use teloxide::types::Sticker;

/// Enumeration representing possible user answer received by the bot.
pub enum Answer {
    String(String),
    Sticker(Sticker),
}

// Struct for packing arguments passed to transition funcitons
pub struct Args {
    pub ans: Answer,
    pub db: std::sync::Arc<tokio::sync::Mutex<RedisConnection>>,
}
