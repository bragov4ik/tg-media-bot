use teloxide::types::{Sticker, InputFile};
use teloxide::prelude::*;
use frunk::Generic;
use crate::dialogue::{Dialogue, Answer};
use crate::logs;

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
            log::info!("{}",
                logs::format_log_chat("Finishing dialogue", cx.chat_id()));
            handle_string(state, cx, text).await;
            exit()
        }
        Answer::Sticker(sticker) => {
            let new_state = ReceiveNamesState {
                sticker,
                ..state
            };
            log::info!("{}",
                logs::format_log_chat("Waiting for names", cx.chat_id()));
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.").await?;
            next(new_state)
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