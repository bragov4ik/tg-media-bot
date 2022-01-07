mod answer;
mod states;

pub use answer::{Answer, Args};

use derive_more::From;
use serde::{Deserialize, Serialize};
use states::{ReceiveNamesState, ReceiveStickerState, ReplacingState};
use teloxide::macros::Transition;

/// Dialogue states.
///
/// Uses `teloxide` dialogue system, see its docs for details and more examples.
#[derive(Clone, Transition, From, Serialize, Deserialize)]
pub enum Dialogue {
    ReceiveSticker(ReceiveStickerState),
    ReceiveNames(ReceiveNamesState),
    Replacing(ReplacingState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Replacing(ReplacingState)
    }
}
