use crate::{commands::Command, utils::log_chat};
use teloxide::{types::MediaKind, utils::command::BotCommand};

/// Enumeration representing possible user answer received by the bot.
pub enum Answer {
    // Any string or unsupported command
    String(String),
    Sticker(teloxide::types::Sticker),
    Command(Command),
}

impl Answer {
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

    pub fn parse(msg: &MediaKind, bot_name: &str, chat_id: i64) -> Option<Answer> {
        // For logging later
        let msg_type: String;

        let result = match &msg {
            MediaKind::Sticker(media) => {
                msg_type = "sticker".to_string();
                Some(Answer::Sticker(media.sticker.clone()))
            }
            other => {
                if let Some(text) = Answer::get_text_from_media(other) {
                    match Command::parse(text, bot_name) {
                        Ok(cmd) => {
                            msg_type = "bot command".to_string();
                            Some(Answer::Command(cmd))
                        }
                        Err(_) => {
                            msg_type = "text (not command)".to_string();
                            Some(Answer::String(text.to_owned()))
                        }
                    }
                } else {
                    msg_type = format!("non-text type ({:?})", other);
                    None
                }
            }
        };
        log_chat!(log::Level::Info, chat_id, "Received a {}", msg_type);
        result
    }
}

// Struct for packing arguments passed to transition funcitons
pub struct Args {
    pub ans: Answer,
    pub db: std::sync::Arc<tokio::sync::Mutex<crate::db::RedisConnection>>,
}
