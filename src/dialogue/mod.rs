mod answer;
mod states;

pub use answer::Answer;

use derive_more::From;
use states::{ReceiveNamesState, ReceiveStickerState, StartState};
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
