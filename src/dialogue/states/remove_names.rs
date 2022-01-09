use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{Answer, Dialogue};
use crate::{utils, RedisConnection};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct RemoveNamesState;

#[teloxide(subtransition)]
async fn remove_names(
    state: RemoveNamesState,
    cx: TransitionIn<AutoSend<Bot>>,
    args: Args,
) -> TransitionOut<Dialogue> {
    let ans: Answer = args.ans;
    match ans {
        Answer::Sticker(_) => {
            log::info!(
                "{}",
                utils::format_log_chat("Waiting for names", cx.chat_id())
            );
            cx.answer(
                "Write aliases you want to remove separated by space or use /cancel to stop.",
            )
            .await?;
            next(state)
        }
        Answer::String(ans_str) => {
            log::info!(
                "{}",
                utils::format_log_chat("Received aliases, removing them...", cx.chat_id())
            );
            remove_aliases(&cx, &ans_str, args.db).await?;
            log::info!(
                "{}",
                utils::format_log_chat("Finished removing aliases", cx.chat_id())
            );
            exit()
        }
        Answer::Command(cmd) => {
            respond_command(&cx, &cmd).await?;
            match cmd {
                Command::Cancel => exit(),
                _ => next(state),
            }
        }
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
                utils::format_log_chat("Ignoring /add at removal stage", cx.chat_id())
            );
            cx.answer("To add new aliases /cancel removal first.").await?;
        }
        Command::Remove => {
            log::info!(
                "{}",
                utils::format_log_chat("Ignoring /remove at removal stage", cx.chat_id())
            );
            cx.answer("Already removing aliases. Type them separated by spaces.")
                .await?;
        }
        Command::Start => {
            log::info!(
                "{}",
                utils::format_log_chat("Printing start message", cx.chat_id())
            );
            handle_start(cx).await?;
        }
        Command::Help => {
            log::info!(
                "{}",
                utils::format_log_chat("Printing help message", cx.chat_id())
            );
            handle_help(cx).await?;
        }
        Command::Cancel => {
            log::info!(
                "{}",
                utils::format_log_chat("Cancelling alias removal", cx.chat_id())
            );
        }
    }
    Ok(())
}

/// Remove aliases received in `text` from database.
/// 
/// Handle the removal, report the result to `cx`.
async fn remove_aliases(
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &String,
    db: Arc<Mutex<RedisConnection>>,
) -> Result<(), teloxide::RequestError> {
    // HashSet lets us omit of repeating removals
    let aliases: HashSet<&str> = text.split_whitespace().collect();
    let mut db = db.lock().await;

    let mut n_removed: i64 = 0;
    let mut fails: Vec<&str> = vec!();

    for &alias in &aliases {
        let res = db.remove_alias(cx.chat_id(), alias).await;
        match res {
            Ok(()) => {
                n_removed += 1;
            }
            Err(_) => {
                fails.push(alias);
            }
        }
    }
    cx.answer(format!("Removed {removed}/{total} (duplicates are omitted)", removed = n_removed, total = aliases.len())).await?;

    // Display failed aliases if needed
    match i64::try_from(aliases.len()) {
        Ok(n_aliases) => {
            if n_removed < n_aliases {
                let mut fails_str = String::new();
                for fail in fails {
                    fails_str.push_str(fail);
                    fails_str.push(' ');
                }
                cx.answer(format!("Aliases that were not removed: {}", fails_str)).await?;    
            }
        }
        Err(e) => {
            log::info!(
                "{}",
                utils::format_log_chat(&format!("Failed converting usize to i64: {}", e), cx.chat_id())
            );
        }
    }
    Ok(())
}
