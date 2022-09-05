use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db::RedisConnection,
    dialogue::{Args, Dialogue, UserInput},
    utils::format_log_chat,
};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::Sticker;
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
    let ans: UserInput = args.input;
    match ans {
        UserInput::Sticker(_) => {
            log::info!("{}", format_log_chat("Waiting for names", cx.chat_id()));
            cx.answer(
                "Sticker was already specified.\
                Write aliases separated by space or use /cancel to stop adding them.",
            )
            .await?;
            next(state)
        }
        UserInput::String(ans_str) => {
            log::info!(
                "{}",
                format_log_chat("Received aliases, saving them...", cx.chat_id())
            );
            save_aliases(&state.sticker, &cx, &ans_str, args.db).await;
            log::info!(
                "{}",
                format_log_chat("Finished saving aliases", cx.chat_id())
            );
            cx.answer("Aliases are set successfully!").await?;
            exit()
        }
        UserInput::Command(cmd) => {
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
            log::info!(
                "{}",
                format_log_chat("Ignoring /add at recieve names stage", cx.chat_id())
            );
            cx.answer("Already adding aliases.").await?;
        }
        Command::Remove => {
            log::info!(
                "{}",
                format_log_chat("Ignoring /remove at removal stage", cx.chat_id())
            );
            cx.answer("To remove aliases /cancel addition first.")
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
                format_log_chat("Cancelling sticker addition", cx.chat_id())
            );
        }
    }
    Ok(())
}

async fn save_aliases(
    sticker: &Sticker,
    cx: &TransitionIn<AutoSend<Bot>>,
    text: &str,
    db: Arc<Mutex<RedisConnection>>,
) {
    let aliases = text.split_whitespace();
    let mut db = db.lock().await;
    for alias in aliases {
        // Maybe it makes sense to create the futures first and then join on them all?
        db.set_alias(cx.chat_id(), alias, &sticker.file_id).await;
    }
}
