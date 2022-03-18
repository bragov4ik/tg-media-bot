use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db::RedisConnection,
    dialogue::{
        states::{AddStickerState, RemoveNamesState},
        Answer, Args, Dialogue,
    },
    utils::{log_chat, log_time},
};
use frunk::Generic;
use regex::Regex;
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
            log_chat!(log::Level::Info, cx.chat_id(), "Waiting for a sticker");
            cx.answer("Send a sticker you want to assign alias to.")
                .await?;
        }
        Command::Remove => {
            log_chat!(log::Level::Info, cx.chat_id(), "Waiting for names to remove");
            cx.answer("Send aliases you want to remove separated by spaces.")
                .await?;
        }
        Command::Start => {
            log_chat!(log::Level::Info, cx.chat_id(), "Printed start message");
            handle_start(cx).await?;
        }
        Command::Help => {
            log_chat!(log::Level::Info, cx.chat_id(), "Printed help message");
            handle_help(cx).await?;
        }
        Command::List => {
            log_chat!(log::Level::Info, cx.chat_id(), "Listing aliases");

            let mut db = db.lock().await;
            if let Some(aliases) = db.get_aliases(cx.chat_id()).await {
                handle_list(cx, aliases).await?;
            }

            log_chat!(log::Level::Info, cx.chat_id(), "Finished listing");
        }
        Command::Cancel => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring cancel in replacing mode");
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
    for alias in extract_aliases(text) {
        let mut db = db.lock().await;
        if let Some(sticker_id) = db.get_sticker_id(chat_id, alias).await {
            stickers.push(InputFile::FileId(sticker_id));
        }
    }
    stickers
}

/// Extract aliases from given text.
///
/// Matches the words with pattern ":<alias>:", returns vector of aliases as result.
///
/// Examples:
/// ":cry:" -> vec!("cry")
/// "sdfssadas  sad fd" -> vec!()
fn extract_aliases(text: &str) -> Vec<&str> {
    if let Ok(r) = Regex::new(":([^:\\s]+):") {
        r.captures_iter(text)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect()
    } else {
        log_time!(log::Level::Error, "Regex for extracting aliases does not compile!");
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_aliases() {
        let cases = vec![
            (":cry:", vec!["cry"]),
            ("inside:cry:text", vec!["cry"]),
            ("inside :cry: text", vec!["cry"]),
            (":cry::not_cry:", vec!["cry", "not_cry"]),
            ("::", vec![]),
            (":😭:", vec!["😭"]),
            (":𝓬𝓻𝔂:", vec!["𝓬𝓻𝔂"]),
        ];
        for (source, target) in cases {
            assert_eq!(extract_aliases(source), target);
        }
    }
}
