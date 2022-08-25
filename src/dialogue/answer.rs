use crate::{commands::Command, utils::log_chat};
use teloxide::{types::MediaKind, utils::command::BotCommand};

/// Enumeration representing possible user messages received by the bot.
pub enum UserInput {
    // Any string or unsupported command
    String(String),
    Sticker(teloxide::types::Sticker),
    Command(Command),
}

// We don't do anything with this data
// (string = name of the type)
pub struct UnsupportedType(String);

impl UserInput {
    fn get_text_from_media(media: &MediaKind) -> Option<&String> {
        match &media {
            MediaKind::Animation(m) => m.caption.as_ref(),
            MediaKind::Audio(m) => m.caption.as_ref(),
            MediaKind::Document(m) => m.caption.as_ref(),
            MediaKind::Photo(m) => m.caption.as_ref(),
            // Maybe extract from polls as well later?
            MediaKind::Poll(_) => None,
            MediaKind::Text(m) => Some(&m.text),
            MediaKind::Video(m) => m.caption.as_ref(),
            MediaKind::Voice(m) => m.caption.as_ref(),
            _ => None,
        }
    }

    pub fn parse(msg: &MediaKind, bot_name: &str, chat_id: i64) -> Result<UserInput, UnsupportedType> {
        let result = match &msg {
            MediaKind::Sticker(media) => {
                UserInput::Sticker(media.sticker.clone())
            }
            other => {
                if let Some(text) = UserInput::get_text_from_media(other) {
                    match Command::parse(text, bot_name) {
                        Ok(cmd) => UserInput::Command(cmd),
                        Err(_) => UserInput::String(text.to_owned())
                    }
                } else {
                    let type_str = format!("{:?}", other);
                    log_chat!(log::Level::Info, chat_id, "Received unsupported message type {}", type_str);
                    return Err(UnsupportedType(type_str));
                }
            }
        };
        let msg_type = match result {
            UserInput::Sticker(_) => "sticker",
            UserInput::Command(_) => "command",
            UserInput::String(_) => "text",
        };
        log_chat!(log::Level::Info, chat_id, "Received a {}", msg_type);
        Ok(result)
    }
}

// Struct for packing arguments passed to transition funcitons
pub struct Args {
    pub ans: UserInput,
    pub db: std::sync::Arc<tokio::sync::Mutex<crate::db_old::RedisConnection>>,
}
