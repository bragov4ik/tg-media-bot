mod dialogue;

use teloxide::{prelude::*};
use teloxide::types::{MessageKind, MediaKind};
use crate::dialogue::{Dialogue, Answer};

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting bot...");
    
    let bot = Bot::from_env().auto_send();
    
    teloxide::repl(bot, |message| async move {
        match message.update.text() {
            Some(text) => log::info!("{}", text),
            None => {},
        }
        message.answer("aboba").await?;
        respond(())
    })
    .await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env().auto_send();

    teloxide::dialogues_repl(bot, |message, dialogue| async move {
        handle_message(message, dialogue).await.expect("Some problem happened")
    });
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
        cx.answer("Send a sticker to start.");
        next(dialogue)
    }

    match &cx.update.kind {
        MessageKind::Common(cmn) => {
            let ans: Answer;
            match &cmn.media_kind {
                MediaKind::Text(media) => {
                    ans = Answer::String(media.text.clone());
                }
                MediaKind::Sticker(media) => {
                    ans = Answer::Sticker(media.sticker.clone());
                }
                _ => {
                    return default_response(cx, dialogue);
                }
            }
            dialogue.react(cx, ans).await
        }
        _ => {
            default_response(cx, dialogue)
        }
    }
}