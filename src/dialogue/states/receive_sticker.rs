use crate::commands::{handle_help, handle_start, Command};
use crate::dialogue::answer::Args;
use crate::dialogue::{
    states::{ReceiveNamesState},
    Answer, Dialogue,
};
use crate::logs;
use frunk::Generic;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

#[derive(Clone, Generic, Serialize, Deserialize)]
pub struct ReceiveStickerState;

#[teloxide(subtransition)]
async fn receive_sticker(
    state: ReceiveStickerState,
    cx: TransitionIn<AutoSend<Bot>>,
    args: Args,
) -> TransitionOut<Dialogue> {
    let ans: Answer = args.ans;
    match ans {
        Answer::Sticker(sticker) => {
            log::info!(
                "{}",
                logs::format_log_chat("Received sticker, waiting for aliases", cx.chat_id())
            );
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.")
                .await?;
            next(ReceiveNamesState::up(state, sticker))
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
                        logs::format_log_chat(
                            "Ignoring text in recieve sticker stage",
                            cx.chat_id()
                        )
                    );
                    cx.answer("Send sticker to assign aliases to or use /cancel.");
                    next(state)
                }
            }
        }
    }

    // let ans: Answer = args.ans;
    // match ans {
    //     Answer::Sticker(sticker) => {
    //         log::info!(
    //             "{}",
    //             logs::format_log_chat("Waiting for names", cx.chat_id())
    //         );
    //         cx.answer("Great! Now specify aliases for the sticker separated by spaces.")
    //             .await?;
    //         next(ReceiveNamesState::up(state, sticker))
    //     }
    //     Answer::String(_) => {
    //         log::info!(
    //             "{}",
    //             logs::format_log_chat("Waiting for a sticker", cx.chat_id())
    //         );
    //         cx.answer("Please send sticker.").await?;
    //         next(state)
    //     }
    // }
}

async fn respond_command(
    cx: &TransitionIn<AutoSend<Bot>>,
    cmd: &Command,
) -> Result<(), teloxide::RequestError> {
    match cmd {
        Command::Add => {
            log::info!(
                "{}",
                logs::format_log_chat("Waiting for a sticker", cx.chat_id())
            );
            cx.answer("Already adding new aliases.").await?;
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
                logs::format_log_chat(
                    "Cancelling alias addition in recieve sticker stage.",
                    cx.chat_id()
                )
            );
            cx.answer("Cancelled alias addition.").await?;
        }
    }
    Ok(())
}
