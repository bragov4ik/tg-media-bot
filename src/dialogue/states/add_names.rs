use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{Answer, Dialogue};
use crate::{utils, RedisConnection};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::Sticker;
use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct AddNamesState {
    pub sticker: Sticker,
}

#[teloxide(subtransition)]
async fn add_names(
    state: AddNamesState,
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
                "Sticker was already specified.\
                Write aliases separated by space or use /cancel to stop adding them.",
            )
            .await?;
            next(state)
        }
        Answer::String(ans_str) => {
            log::info!(
                "{}",
                utils::format_log_chat("Received aliases, saving them...", cx.chat_id())
            );
            save_aliases(&state.sticker, &cx, &ans_str, args.db).await;
            log::info!(
                "{}",
                utils::format_log_chat("Finished saving aliases", cx.chat_id())
            );
            log::info!(
                "{}",
                utils::format_log_chat("Finishing dialogue", cx.chat_id())
            );
            cx.answer("Aliases are set successfully!").await?;
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
                utils::format_log_chat("Ignoring /add at recieve names stage", cx.chat_id())
            );
            cx.answer("Already adding aliases.").await?;
        }
        Command::Remove => {
            log::info!(
                "{}",
                utils::format_log_chat("Ignoring /remove at removal stage", cx.chat_id())
            );
            cx.answer("To remove aliases /cancel addition first.")
                .await?;
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
                utils::format_log_chat("Cancelling sticker addition", cx.chat_id())
            );
        }
    }
    Ok(())
}

async fn save_aliases(
    sticker: &Sticker,
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &String,
    db: Arc<Mutex<RedisConnection>>,
) {
    let aliases = text.split_whitespace();
    let mut db = db.lock().await;
    for alias in aliases {
        // Maybe it makes sense to create the futures first and then join on them all?
        db.set_alias(cx.chat_id(), alias, &sticker.file_id)
        .await;
    }
}
