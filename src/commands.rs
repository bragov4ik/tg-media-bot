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
    Cancel,
}

/// Write start message in given context.
pub async fn handle_start(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), teloxide::RequestError> {
    cx.answer(
        "Hello, I send stickers when I see their specified \
    names in messages. To assign an alias to the sticker write /add. \
    For more info use /help. 
    
    Note: I can properly work in groups only if given admin permissions, \
    otherwise messages can't be seen.",
    )
    .await?;
    Ok(())
}

/// Write help message in given context.
pub async fn handle_help(
    cx: &UpdateWithCx<AutoSend<Bot>, Message>,
) -> Result<(), teloxide::RequestError> {
    cx.answer(
        "Commands:\n
    /add - add new alias to sticker
    /cancel - cancel addition process
    /start - show start message
    /help - show this message",
    )
    .await?;
    Ok(())
}
