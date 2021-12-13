mod dialogue;
mod logs;
mod db;

use crate::dialogue::{Answer, Dialogue};
use teloxide::prelude::*;
use teloxide::types::{MediaKind, MessageKind};

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env().auto_send();

    teloxide::dialogues_repl(bot, |message, dialogue| async move {
        handle_message(message, dialogue)
            .await
            .expect("Some problem happened")
    })
    .await;
    log::info!("Closing the bot...");
}

async fn handle_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    // Don't know hot to avoid repeating of this code properly
    fn default_response(
        cx: UpdateWithCx<AutoSend<Bot>, Message>,
        dialogue: Dialogue,
    ) -> TransitionOut<Dialogue> {
        log::info!(
            "{}",
            logs::format_log_chat("Received something else", cx.chat_id())
        );
        cx.answer("Send a sticker to start.");
        next(dialogue)
    }

    match &cx.update.kind {
        MessageKind::Common(cmn) => {
            let ans: Answer;
            match &cmn.media_kind {
                MediaKind::Text(media) => {
                    log::info!("{}", logs::format_log_chat("Received a text", cx.chat_id()));
                    ans = Answer::String(media.text.clone());
                }
                MediaKind::Sticker(media) => {
                    log::info!(
                        "{}",
                        logs::format_log_chat("Received a sticker", cx.chat_id())
                    );
                    ans = Answer::Sticker(media.sticker.clone());
                }
                _ => {
                    return default_response(cx, dialogue);
                }
            }
            let res = dialogue.react(cx, ans).await;
            res
        }
        _ => default_response(cx, dialogue),
    }
}
