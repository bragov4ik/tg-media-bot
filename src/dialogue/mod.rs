mod states;
mod answer;

pub use answer::Answer;

use states::{StartState, ReceiveNamesState, ReceiveStickerState};
use derive_more::From;
use teloxide::macros::Transition;

#[derive(Clone, Transition, From)]
pub enum Dialogue {
    ReceiveSticker(ReceiveStickerState),
    ReceiveNames(ReceiveNamesState),
    Start(StartState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}