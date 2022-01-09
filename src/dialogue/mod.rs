mod answer;
mod states;

pub use answer::{Answer, Args};

use derive_more::From;
use serde::{Deserialize, Serialize};
use states::{AddNamesState, AddStickerState, ReplacingState};
use teloxide::macros::Transition;

/// Dialogue states.
///
/// Uses `teloxide` dialogue system, see its docs for details and more examples.
#[derive(Clone, Transition, From, Serialize, Deserialize)]
pub enum Dialogue {
    AddSticker(AddStickerState),
    AddNames(AddNamesState),
    Replacing(ReplacingState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Replacing(ReplacingState)
    }
}
