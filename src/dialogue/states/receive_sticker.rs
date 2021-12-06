use teloxide::prelude::*;
use frunk::Generic;
use crate::dialogue::{Dialogue, Answer, states::ReceiveNamesState};
use crate::logs;

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
            log::info!("{}",
                logs::format_log_chat("Waiting for names", cx.chat_id()));
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.").await?;
            next(ReceiveNamesState::up(state, sticker))
        }
        Answer::String(_) => {
            log::info!("{}",
                logs::format_log_chat("Waiting for a sticker", cx.chat_id()));
            cx.answer("Please send sticker.").await?;
            next(state)
        }
    }
}