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
    match ans {
        Answer::Sticker(sticker) => {
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.").await?;
            next(ReceiveNamesState::up(state, sticker))
        }
        Answer::String(_) => {
            cx.answer("Please send sticker.").await?;
            next(state)
        }
    }
}