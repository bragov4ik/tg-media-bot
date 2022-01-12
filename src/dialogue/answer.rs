/// Enumeration representing possible user answer received by the bot.
pub enum Answer {
    // Any string or unsupported command
    String(String),
    Sticker(teloxide::types::Sticker),
    Command(crate::commands::Command),
}

// Struct for packing arguments passed to transition funcitons
pub struct Args {
    pub ans: Answer,
    pub db: std::sync::Arc<tokio::sync::Mutex<crate::db::RedisConnection>>,
}
