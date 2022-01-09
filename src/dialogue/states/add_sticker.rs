use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{states::AddNamesState, Answer, Dialogue};
use crate::utils;
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

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
            log::info!(
                "{}",
                utils::format_log_chat("Received sticker, waiting for aliases", cx.chat_id())
            );
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.")
                .await?;
            next(AddNamesState::up(state, sticker))
        }
        Answer::String(_) => {
            log::info!(
                "{}",
                utils::format_log_chat(
                    "Ignoring text in recieve sticker stage",
                    cx.chat_id()
                )
            );
            cx.answer("Send sticker to assign aliases to or use /cancel.").await?;
            next(state)
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
                utils::format_log_chat("Waiting for a sticker", cx.chat_id())
            );
            cx.answer("Already adding new aliases.").await?;
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
                utils::format_log_chat(
                    "Cancelling alias addition in recieve sticker stage.",
                    cx.chat_id()
                )
            );
            cx.answer("Cancelled alias addition.").await?;
        }
    }
    Ok(())
}
