use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{Answer, Dialogue};
use crate::{utils, RedisConnection};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
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
            remove_aliases(&cx, &ans_str, args.db).await;
            log::info!(
                "{}",
                utils::format_log_chat("Finished removing aliases", cx.chat_id())
            );
            cx.answer("Aliases have been removed successfully!").await?;
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
                utils::format_log_chat("Ignoring /add at deletion stage", cx.chat_id())
            );
            cx.answer("To add new aliases /cancel removal first.").await?;
        }
        Command::Start => {
            log::info!(
                "{}",
                utils::format_log_chat("Printed start message", cx.chat_id())
            );
            handle_start(cx).await?;
        }
        Command::Help => {
            log::info!(
                "{}",
                utils::format_log_chat("Printed help message", cx.chat_id())
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


async fn remove_aliases(
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &String,
    db: Arc<Mutex<RedisConnection>>,
) {
    let aliases = text.split_whitespace();
    let mut db = db.lock().await;
    for alias in aliases {
        db.remove_alias(cx.chat_id(), alias).await;
    }
}
