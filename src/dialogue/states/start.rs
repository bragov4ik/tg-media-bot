use teloxide::prelude::*;
use crate::dialogue::{Dialogue, Answer, states::ReceiveStickerState};
use crate::logs;

#[derive(Clone)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: Answer,
) -> TransitionOut<Dialogue> {
    log::info!("{}",
        logs::format_log_chat("Waiting for a sticker", cx.chat_id()));
    cx.answer("To start send any sticker.").await?;
    next(ReceiveStickerState)
}