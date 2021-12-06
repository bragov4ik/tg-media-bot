use teloxide::prelude::*;
use frunk::Generic;
use crate::dialogue::{Dialogue, Answer, states::ReceiveNamesState};

#[derive(Clone, Generic)]
pub struct ReceiveStickerState;

#[teloxide(subtransition)]
async fn receive_sticker(
    state: ReceiveStickerState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: Answer,
) -> TransitionOut<Dialogue>{
    cx.answer("Great! Now specify aliases for the sticker separated by spaces.");
    next(ReceiveNamesState::up(state, ans))
}