use teloxide::types::{Sticker, InputFile};
use teloxide::prelude::*;
use frunk::Generic;
use crate::dialogue::{Dialogue, Answer};

#[derive(Clone, Generic)]
pub struct ReceiveNamesState {
    pub sticker: Sticker,
}

#[teloxide(subtransition)]
async fn receive_names(
    state: ReceiveNamesState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: Answer,
) -> TransitionOut<Dialogue> {
    match ans {
        Answer::String(text) =>  {
            handle_string(state, cx, text);
            exit()
        }
        Answer::Sticker(sticker) => {
            let newState = ReceiveNamesState {
                sticker,
                ..state
            };
            next(newState)
        }
    }
}

async fn handle_string(
    state: ReceiveNamesState,
    cx: TransitionIn<AutoSend<Bot>>,
    text: String
) {
    cx.answer(text).await.expect("Failed to echo aliases back");
    cx.answer_sticker(InputFile::FileId(state.sticker.file_id)).await.expect("Failed to echo stickers back");
}