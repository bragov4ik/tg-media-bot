use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{Answer, Dialogue};
use crate::{logs, RedisConnection};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::{Sticker};
use teloxide::utils::command::BotCommand;

use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct ReceiveNamesState {
    pub sticker: Sticker,
}

#[teloxide(subtransition)]
async fn receive_names(
    state: ReceiveNamesState,
    cx: TransitionIn<AutoSend<Bot>>,
    args: Args,
) -> TransitionOut<Dialogue> {
    let ans: Answer = args.ans;
    match ans {
        Answer::Sticker(_) => {
            log::info!(
                "{}",
                logs::format_log_chat("Waiting for names", cx.chat_id())
            );
            cx.answer(
                "Sticker was already specified.\
                Write aliases separated by space or use /cancel to stop adding them.",
            )
            .await?;
            next(state)
        }
        Answer::String(ans_str) => {
            match Command::parse(&ans_str, "") {
                Ok(cmd) => {
                    // Command is received
                    respond_command(&cx, &cmd).await?;
                    match cmd {
                        Command::Cancel => exit(),
                        _ => next(state),
                    }
                }
                Err(_) => {
                    // We got simple text
                    log::info!(
                        "{}",
                        logs::format_log_chat("Received aliases, saving them...", cx.chat_id())
                    );
                    save_aliases(&state.sticker, &cx, &ans_str, args.db).await;
                    log::info!(
                        "{}",
                        logs::format_log_chat("Finished saving aliases", cx.chat_id())
                    );
                    log::info!(
                        "{}",
                        logs::format_log_chat("Finishing dialogue", cx.chat_id())
                    );
                    cx.answer("Aliases are set successfully!").await?;
                    exit()
                }
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
                logs::format_log_chat("Ignoring /add at recieve names stage", cx.chat_id())
            );
            cx.answer("Already adding aliases.").await?;
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
                logs::format_log_chat("Cancelling sticker addition", cx.chat_id())
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
    db.set_aliases(cx.chat_id(), aliases, &sticker.file_id)
        .await;
}
