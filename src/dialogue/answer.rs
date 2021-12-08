use teloxide::types::Sticker;

// Enumeration representing possible user answer received by the bot.
pub enum Answer {
    String(String),
    Sticker(Sticker),
}
