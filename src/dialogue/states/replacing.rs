use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db::RedisConnection,
    dialogue::{
        states::{AddStickerState, RemoveNamesState},
        Answer, Args, Dialogue,
    },
    utils::format_log_chat,
};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::InputFile;
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
    match ans {
        Answer::String(ans_str) => {
            handle_replace(&cx, &ans_str, args.db).await?;
            next(state)
        }
        Answer::Command(cmd) => {
            respond_command(&cx, &cmd, args.db).await?;
            match cmd {
                Command::Add => next(AddStickerState),
                Command::Remove => next(RemoveNamesState),
                _ => next(state),
            }
        }
        Answer::Sticker(_) => next(state),
    }
}

async fn respond_command(
    cx: &TransitionIn<AutoSend<Bot>>,
    cmd: &Command,
    db: Arc<Mutex<RedisConnection>>,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Add => {
            log::info!("{}", format_log_chat("Waiting for a sticker", cx.chat_id()));
            cx.answer("Send a sticker you want to assign alias to.")
                .await?;
        }
        Command::Remove => {
            log::info!(
                "{}",
                format_log_chat("Waiting for names to remove", cx.chat_id())
            );
            cx.answer("Send aliases you want to remove separated by spaces.")
                .await?;
        }
        Command::Start => {
            log::info!("{}", format_log_chat("Printed start message", cx.chat_id()));
            handle_start(cx).await?;
        }
        Command::Help => {
            log::info!("{}", format_log_chat("Printed help message", cx.chat_id()));
            handle_help(cx).await?;
        }
        Command::List => {
            log::info!("{}", format_log_chat("Listing aliases", cx.chat_id()));

            let mut db = db.lock().await;
            if let Some(aliases) = db.get_aliases(cx.chat_id()).await {
                handle_list(cx, aliases).await?;
            }

            log::info!("{}", format_log_chat("Finished listing", cx.chat_id()));
        }
        Command::Cancel => {
            log::info!(
                "{}",
                format_log_chat("Ignoring cancel in replacing mode", cx.chat_id())
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
    Ok(())
}

async fn extract_stickers(
    text: &str,
    chat_id: i64,
    db: Arc<Mutex<RedisConnection>>,
) -> Vec<InputFile> {
    let mut stickers: Vec<InputFile> = Vec::new();
    for word in text.split_whitespace() {
        if let Some(alias) = parse_alias(word) {
            let mut db = db.lock().await;
            if let Some(sticker_id) = db.get_sticker_id(chat_id, alias).await {
                stickers.push(InputFile::FileId(sticker_id));
            }
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
    word.strip_prefix(':')?.strip_suffix(':')
}
