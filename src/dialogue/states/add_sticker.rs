use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db_old::RedisConnection,
    dialogue::{states::AddNamesState, Answer, Args, Dialogue},
    utils::log_chat,
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
    let ans: Answer = args.ans;
    match ans {
        Answer::Sticker(sticker) => {
            log_chat!(log::Level::Info, cx.chat_id(), "Received sticker, waiting for aliases");
            cx.answer(
                "Great! Now specify aliases for the sticker \
                separated by spaces (without colons!).",
            )
            .await?;
            next(AddNamesState::up(state, sticker))
        }
        Answer::String(_) => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring text in recieve sticker stage");
            cx.answer("Send sticker to assign aliases to or use /cancel.")
                .await?;
            next(state)
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
            log_chat!(log::Level::Info, cx.chat_id(), "Waiting for a sticker");
            cx.answer("Already adding new aliases.").await?;
        }
        Command::Remove => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring /remove at adding stage");
            cx.answer("To remove aliases /cancel addition first.")
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
            log_chat!(
                log::Level::Info, cx.chat_id(), 
                "Cancelling alias addition in recieve sticker stage."
            );
            cx.answer("Cancelled alias addition.").await?;
        }
    }
    Ok(())
}
