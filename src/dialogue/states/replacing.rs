use crate::db::RedisConnection;
use crate::dialogue::answer::Args;
use crate::dialogue::{states::ReceiveStickerState, Answer, Dialogue};
use crate::{
    commands::{handle_help, handle_start, Command},
    logs,
};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::utils::command::BotCommand;

use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct ReplacingState;

#[teloxide(subtransition)]

async fn replacing_state(
    state: ReplacingState,
    cx: TransitionIn<AutoSend<Bot>>,
    args: Args,
) -> TransitionOut<Dialogue> {
    let ans: Answer = args.ans;
    if let Answer::String(ans_str) = ans {
        match Command::parse(&ans_str, "") {
            Ok(cmd) => {
                respond_command(&cx, &cmd).await?;
                match cmd {
                    Command::Add => next(ReceiveStickerState),
                    _ => next(state),
                }
            }
            Err(_) => {
                handle_replace(&cx, &ans_str, args.db).await?;
                next(state)
            }
        }
    } else {
        next(state)
    }
}

async fn respond_command(
    cx: &TransitionIn<AutoSend<Bot>>,
    cmd: &Command,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Add => {
            log::info!(
                "{}",
                logs::format_log_chat("Waiting for a sticker", cx.chat_id())
            );
            cx.answer("Send a sticker you want to assign alias to.")
                .await?;
        }
        Command::Start => {
            log::info!(
                "{}",
                logs::format_log_chat("Printed start message", cx.chat_id())
            );
            handle_start(cx).await?;
        }
        Command::Help => {
            log::info!(
                "{}",
                logs::format_log_chat("Printed help message", cx.chat_id())
            );
            handle_help(cx).await?;
        }
        Command::Cancel => {
            log::info!(
                "{}",
                logs::format_log_chat("Ignoring cancel in replacing mode", cx.chat_id())
            );
        }
    }
    Ok(())
}

async fn handle_replace(
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &str,
    db: Arc<Mutex<RedisConnection>>,
) -> Result<(), teloxide::RequestError> {
    let stickers = extract_stickers(text, cx.chat_id(), db).await;
    for sticker in stickers {
        cx.answer_sticker(sticker).await?;
    }
    cx.answer("aboba").await?;
    Ok(())
}

async fn extract_stickers(
    text: &str,
    chat_id: i64,
    db: Arc<Mutex<RedisConnection>>,
) -> Vec<InputFile> {
    let mut stickers: Vec<InputFile> = Vec::new();
    for word in text.split_whitespace() {
        match parse_alias(word) {
            Some(alias) => {
                let mut db = db.lock().await;
                if let Some(sticker_id) = db.get_sticker_id(chat_id, alias).await {
                    stickers.push(InputFile::FileId(sticker_id));
                }
            }
            None => {}
        }
    }
    stickers
}

/// Parse given text as sticker alias.
///
/// Matches the word with pattern ":<alias>:", returns <alias> as result. If the word does not fit the format,
/// returns `None`.
///
/// Examples:
/// ":cry:" -> Some("cry")
/// "sdfs:::fd" -> None
fn parse_alias(word: &str) -> Option<&str> {
    word.strip_prefix(":")?.strip_suffix(":")
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse_alias(":aboba:"), Some("aboba"));
        assert_eq!(parse_alias("abeba:"), None);
        assert_eq!(parse_alias(":aboeba"), None);
        assert_eq!(parse_alias(":::sda as;dask121343aboeba"), None);
    }
}
