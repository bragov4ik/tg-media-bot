mod answer;
mod states;

pub use answer::Answer;

use derive_more::From;
use serde::{Deserialize, Serialize};
use states::{ReceiveNamesState, ReceiveStickerState, StartState};
use teloxide::macros::Transition;

/// Dialogue states.
/// 
/// Uses `teloxide` dialogue system, see its docs for details and more examples.
#[derive(Clone, Transition, From, Serialize, Deserialize)]
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
