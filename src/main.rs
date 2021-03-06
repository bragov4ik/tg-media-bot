mod commands;
mod db;
mod dialogue;
mod utils;

use crate::db::RedisConnection;
use crate::dialogue::Dialogue;
use crate::utils::format_log_chat;
use std::sync::Arc;
use teloxide::prelude::*;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    run().await;
}

/// Main run function.
///
/// Sets everything up and starts the bot.
async fn run() {
    use tokio_stream::wrappers::UnboundedReceiverStream;

    teloxide::enable_logging!();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env().auto_send();

    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(args);

    let db_shared: Arc<Mutex<RedisConnection>> = Arc::new(Mutex::new(
        match db::RedisConnection::new(&config.redis_ip[..]).await {
            Ok(v) => v,
            Err(err) => panic!("Could not start redis connection: {}", err),
        },
    ));

    Dispatcher::new(bot)
        .messages_handler(
            |rx: UnboundedReceiver<UpdateWithCx<AutoSend<Bot>, Message>>| async move {
                UnboundedReceiverStream::new(rx)
                    .for_each_concurrent(None, |cx| async {
                        handle_message(cx, db_shared.clone()).await
                    })
                    .await;
            },
        )
        .dispatch()
        .await;
    log::info!("Closing the bot...");
}

/// Application configuration.
///
/// Contains info necessary for running the bot, such as IP address of redis
#[derive(PartialEq, Debug)]
struct Config {
    redis_ip: String,
}

/// Parse config from splitted arguments.
///
/// Assumes `std::env::args().collect()` ordering and length of vector.
fn parse_args(args: Vec<String>) -> Config {
    match args.len() {
        1 => Config {
            redis_ip: String::from("redis://127.0.0.1/"),
        },
        2 => Config {
            redis_ip: String::from("redis://") + &args[1][..] + "/",
        },
        _ => {
            print_usage();
            panic!();
        }
    }
}

/// Print out usage of the application in standard output
fn print_usage() {
    println!(
        "Telegram bot. Run with no arguments or specify redis ip as first argument \
    (without 'redis://' prefix)."
    )
}

/// Handle message update in context of dialogue.
///
/// Log special cases such as receiving text or sticker, prepare
/// and provide `Answer` argument for dialogue.
///
/// Returns result of the handling that contains `DialogueStage`
/// (uses `teloxide` dialogues, find details there).
async fn handle_dialogue(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
    db: Arc<Mutex<RedisConnection>>,
) -> TransitionOut<Dialogue> {
    use crate::commands::Command;
    use crate::dialogue::Answer;
    use teloxide::types::{MediaKind, MessageKind};
    use teloxide::utils::command::BotCommand;

    // Don't know hot to avoid repeating of this code properly
    async fn default_response(
        cx: UpdateWithCx<AutoSend<Bot>, Message>,
        dialogue: Dialogue,
    ) -> TransitionOut<Dialogue> {
        log::info!(
            "{}",
            format_log_chat("Received something else", cx.chat_id())
        );
        next(dialogue)
    }

    match &cx.update.kind {
        MessageKind::Common(cmn) => {
            // Parse `Answer` with logging the process.
            let ans: Answer;
            match &cmn.media_kind {
                MediaKind::Text(media) => {
                    let bot_info: teloxide::types::Me =
                        cx.requester.inner().get_me().send().await?;
                    ans = match Command::parse(
                        &media.text,
                        bot_info.user.username.unwrap_or_default(),
                    ) {
                        Ok(cmd) => {
                            log::info!(
                                "{}",
                                format_log_chat("Received a bot command", cx.chat_id())
                            );
                            Answer::Command(cmd)
                        }
                        Err(_) => {
                            log::info!(
                                "{}",
                                format_log_chat(
                                    "Received a text or unsupported command",
                                    cx.chat_id()
                                )
                            );
                            Answer::String(media.text.clone())
                        }
                    };
                }
                MediaKind::Sticker(media) => {
                    log::info!("{}", format_log_chat("Received a sticker", cx.chat_id()));
                    ans = Answer::Sticker(media.sticker.clone());
                }
                _ => {
                    return default_response(cx, dialogue).await;
                }
            }

            // Forward the user answer to dialogue to handle.
            let args = crate::dialogue::Args { ans, db };
            dialogue.react(cx, args).await
        }
        _ => default_response(cx, dialogue).await,
    }
}

/// Handle message update.
///
/// Find `Dialogue` for `handle_dialogue` from db. Use the function
/// result to update dialogue state in database.
async fn handle_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    db_shared: Arc<Mutex<RedisConnection>>,
) {
    let mut db_con: tokio::sync::MutexGuard<RedisConnection> = db_shared.lock().await;

    // Obtain dialogue from database
    let chat_id = cx.update.chat_id();
    let from_id = cx.update.from().map(|u| u.id);
    let dialogue: Dialogue = match db_con
        .get_dialogue(chat_id, from_id)
        .await
        .map(Option::unwrap_or_default)
    {
        Ok(d) => d,
        Err(e) => {
            log::error!(
                "{}",
                format_log_chat(
                    &format!(
                        "Could not get dialogue (from {f:?}): {e:?}",
                        f = from_id,
                        e = e
                    ),
                    chat_id
                )
            );
            return;
        }
    };
    drop(db_con);

    // Handle the dialogue and receive results.
    let stage = match handle_dialogue(cx, dialogue, db_shared.clone()).await {
        Ok(a) => a,
        Err(e) => {
            log::error!(
                "{}",
                format_log_chat(
                    &format!(
                        "Could not handle dialogue (from {f:?}): {e:?}",
                        f = from_id,
                        e = e
                    ),
                    chat_id
                )
            );
            return;
        }
    };

    let mut db_con: tokio::sync::MutexGuard<RedisConnection> = db_shared.lock().await;
    // Update the dialogue state in database.
    match stage {
        DialogueStage::Next(new_dialogue) => {
            if let Err(e) = db_con.update_dialogue(chat_id, from_id, new_dialogue).await {
                log::error!("Storage::update_dialogue failed: {:?}", e);
            }
        }
        DialogueStage::Exit => {
            if let Err(e) = db_con.remove_dialogue(chat_id, from_id).await {
                log::error!("Storage::remove_dialogue failed: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let args = vec!["asdsad".to_owned()];
        assert_eq!(
            parse_args(args),
            Config {
                redis_ip: "redis://127.0.0.1/".to_owned()
            }
        );

        let args = vec!["asdsad".to_owned(), "192.168.88.123".to_owned()];
        assert_eq!(
            parse_args(args),
            Config {
                redis_ip: format!("redis://{}/", "192.168.88.123").to_owned()
            }
        );
    }
}
