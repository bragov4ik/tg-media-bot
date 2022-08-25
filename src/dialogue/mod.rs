mod answer;
mod states;

pub use answer::{UserInput, Args};
use derive_more::From;
use serde::{Deserialize, Serialize};
use states::{AddNamesState, AddStickerState, RemoveNamesState, ReplacingState};
use teloxide::macros::Transition;

/// Dialogue states.
///
/// Uses `teloxide` dialogue system, see its docs for details and more examples.
#[allow(clippy::large_enum_variant)] // because `Box` doesn't work with `Transition`
#[derive(Clone, Transition, From, Serialize, Deserialize)]
pub enum Dialogue {
    AddSticker(AddStickerState),
    AddNames(AddNamesState),
    RemoveNames(RemoveNamesState),
    Replacing(ReplacingState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Replacing(ReplacingState)
    }
}
