use crate::dialogue::{Answer, Dialogue};
use crate::logs;
use frunk::Generic;
use teloxide::prelude::*;
use teloxide::types::{InputFile, Sticker};

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
        Answer::String(text) => {
            log::info!(
                "{}",
                logs::format_log_chat("Finishing dialogue", cx.chat_id())
            );
            handle_string(state, cx, text).await;
            exit()
        }
        Answer::Sticker(sticker) => {
            let new_state = ReceiveNamesState { sticker };
            log::info!(
                "{}",
                logs::format_log_chat("Waiting for names", cx.chat_id())
            );
            cx.answer("Great! Now specify aliases for the sticker separated by spaces.")
                .await?;
            next(new_state)
        }
    }
}

async fn handle_string(state: ReceiveNamesState, cx: TransitionIn<AutoSend<Bot>>, text: String) {
    let keys_iter = text.split_whitespace();
    for _key in keys_iter {
        cx.answer(_key).await.expect("Failed to echo aliases back");
    }
    cx.answer(text).await.expect("Failed to echo aliases back");
    cx.answer_sticker(InputFile::FileId(state.sticker.file_id))
        .await
        .expect("Failed to echo stickers back");
}
