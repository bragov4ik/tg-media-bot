use teloxide::prelude::*;
use crate::dialogue::{Dialogue, Answer, states::ReceiveStickerState};

#[derive(Clone)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: Answer,
) -> TransitionOut<Dialogue> {
    cx.answer("To start send any sticker.").await?;
    next(ReceiveStickerState)
}