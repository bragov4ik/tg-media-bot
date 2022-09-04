use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db::RedisConnection,
    dialogue::{states::AddNamesState, UserInput, Args, Dialogue},
    utils::format_log_chat,
};
use frunk::Generic;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct AddStickerState;

#[teloxide(subtransition)]
async fn add_sticker(
    state: AddStickerState,
    cx: TransitionIn<AutoSend<Bot>>,
    args: Args,
) -> TransitionOut<Dialogue> {
    let ans: UserInput = args.input;
    match ans {
        UserInput::Sticker(sticker) => {
            log::info!(
                "{}",
                format_log_chat("Received sticker, waiting for aliases", cx.chat_id())
            );
            cx.answer(
                "Great! Now specify aliases for the sticker \
                separated by spaces (without colons!).",
            )
            .await?;
            next(AddNamesState::up(state, sticker))
        }
        UserInput::String(_) => {
            log::info!(
                "{}",
                format_log_chat("Ignoring text in recieve sticker stage", cx.chat_id())
            );
            cx.answer("Send sticker to assign aliases to or use /cancel.")
                .await?;
            next(state)
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
            log::info!("{}", format_log_chat("Waiting for a sticker", cx.chat_id()));
            cx.answer("Already adding new aliases.").await?;
        }
        Command::Remove => {
            log::info!(
                "{}",
                format_log_chat("Ignoring /remove at adding stage", cx.chat_id())
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
                format_log_chat(
                    "Cancelling alias addition in recieve sticker stage.",
                    cx.chat_id()
                )
            );
            cx.answer("Cancelled alias addition.").await?;
        }
    }
    Ok(())
}
