use crate::dialogue::{states::ReceiveStickerState, Answer, Dialogue};
use crate::logs;
use teloxide::prelude::*;
use serde::{ Serialize, Deserialize };

#[derive(Clone, Serialize, Deserialize)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: Answer,
) -> TransitionOut<Dialogue> {
    log::info!(
        "{}",
        logs::format_log_chat("Waiting for a sticker", cx.chat_id())
    );
    cx.answer("To start send any sticker.").await?;
    next(ReceiveStickerState)
}
