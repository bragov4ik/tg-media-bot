//! Telegram commands.
//!
//! Defines all available commands and gives implementations for some of them.
use teloxide::prelude::{AutoSend, Bot, Message, UpdateWithCx};
use teloxide::utils::command::BotCommand;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase")]
pub enum Command {
    Start,
    Help,
    Add,
    Remove,
    Cancel,
}

/// Write start message in given context.
pub async fn handle_start(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), teloxide::RequestError> {
    cx.answer(
        "Hello, I send stickers when I see their specified \
    names in messages.\n\
    To assign an alias to the sticker write /add and follow instructions. \n\
    Then put an alias inside colons  (:alias:) inside a message and bot will\
    send associated sticker.\n\
    For more info and commands see /help. \n\n\
    Note: I can properly work in groups only if given admin permissions, \
    otherwise I can't see most messages (apart from bot commands, mentions\
    , replies).",
    )
    .await?;
    Ok(())
}

/// Write help message in given context.
pub async fn handle_help(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), teloxide::RequestError> {
    cx.answer(
        "Commands:\n\
    /add - add new alias to sticker\n\
    /remove - remove aliases\n\
    /cancel - cancel addition process\n\
    /start - show start message\n\
    /help - show this message",
    )
    .await?;
    Ok(())
}
