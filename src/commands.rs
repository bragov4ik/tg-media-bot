//! Telegram commands.
//! 
//! Defines all available commands and gives implementations for some of them.
use teloxide::prelude::{UpdateWithCx, AutoSend, Bot, Message};
use teloxide::{utils::command::BotCommand};

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase")]
enum Command {
    Start,
    Help,
    Add,
    Cancel,
}

pub fn handle_start(cx: UpdateWithCx<AutoSend<Bot>, Message>) {
    cx.answer("Hello, I send stickers when I see their specified \
    names in messages. To assign an alias to the sticker write /add. \
    For more info use /help. 
    
    Note: I can properly work in groups only if given admin permissions, \
    otherwise messages can't be seen.");
}

pub fn handle_help(cx: UpdateWithCx<AutoSend<Bot>, Message>) {
    cx.answer("Commands:\n
    /add - add new alias to sticker
    /cancel - cancel addition process
    /start - show start message
    /help - show this message");
}