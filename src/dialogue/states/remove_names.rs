use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db_old::RedisConnection,
    dialogue::{Answer, Args, Dialogue},
    utils::log_chat,
};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
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
            log_chat!(log::Level::Info, cx.chat_id(), "Waiting for names");
            cx.answer(
                "Write aliases you want to remove separated by space or use /cancel to stop.",
            )
            .await?;
            next(state)
        }
        Answer::String(ans_str) => {
            log_chat!(log::Level::Info, cx.chat_id(), "Received aliases, removing them...");
            remove_aliases(&cx, &ans_str, args.db).await?;
            log_chat!(log::Level::Info, cx.chat_id(), "Finished removing aliases");
            exit()
        }
        Answer::Command(cmd) => {
            respond_command(&cx, &cmd, args.db).await?;
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
    db: Arc<Mutex<RedisConnection>>,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Add => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring /add at removal stage");
            cx.answer("To add new aliases /cancel removal first.")
                .await?;
        }
        Command::Remove => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring /remove at removal stage");
            cx.answer("Already removing aliases. Type them separated by spaces.")
                .await?;
        }
        Command::Start => {
            log_chat!(log::Level::Info, cx.chat_id(), "Printing start message");
            handle_start(cx).await?;
        }
        Command::Help => {
            log_chat!(log::Level::Info, cx.chat_id(), "Printing help message");
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
            log_chat!(log::Level::Info, cx.chat_id(), "Cancelling alias removal");
        }
    }
    Ok(())
}

/// Remove aliases received in `text` from database.
///
/// Handle the removal, report the result to `cx`.
async fn remove_aliases(
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &str,
    db: Arc<Mutex<RedisConnection>>,
) -> Result<(), teloxide::RequestError> {
    use std::collections::HashSet;
    // HashSet lets us omit repeating removals
    let aliases: HashSet<&str> = text.split_whitespace().collect();
    let mut db = db.lock().await;

    let mut n_removed: i64 = 0;
    let mut fails: Vec<&str> = vec![];

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
    cx.answer(format!(
        "Removed {removed}/{total} (duplicates are omitted)",
        removed = n_removed,
        total = aliases.len()
    ))
    .await?;

    // Display failed aliases if needed
    match i64::try_from(aliases.len()) {
        Ok(n_aliases) => {
            if n_removed < n_aliases {
                let mut fails_str = String::new();
                for fail in fails {
                    fails_str.push_str(fail);
                    fails_str.push(' ');
                }
                cx.answer(format!("Aliases that were not removed: {}", fails_str))
                    .await?;
            }
        }
        Err(e) => {
            log_chat!(log::Level::Error, cx.chat_id(), "Failed converting usize to i64: {}", e);
        }
    }
    Ok(())
}
