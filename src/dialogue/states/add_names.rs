use crate::{
    commands::{handle_help, handle_list, handle_start, Command},
    db::RedisConnection,
    dialogue::{Answer, Args, Dialogue},
    utils::log_chat,
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
    let ans: Answer = args.ans;
    match ans {
        Answer::Sticker(_) => {
            log_chat!(log::Level::Info, cx.chat_id(), "Waiting for names");
            cx.answer(
                "Sticker was already specified.\
                Write aliases separated by space or use /cancel to stop adding them.",
            )
            .await?;
            next(state)
        }
        Answer::String(ans_str) => {
            log_chat!(log::Level::Info, cx.chat_id(), "Received aliases, saving them...");
            save_aliases(&state.sticker, &cx, &ans_str, args.db).await;
            log_chat!(log::Level::Info, cx.chat_id(), "Finished saving aliases");
            cx.answer("Aliases are set successfully!").await?;
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
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring /add at recieve names stage");
            cx.answer("Already adding aliases.").await?;
        }
        Command::Remove => {
            log_chat!(log::Level::Info, cx.chat_id(), "Ignoring /remove at removal stage");
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
            log_chat!(log::Level::Info, cx.chat_id(), "Cancelling sticker addition");
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
