mod dialogue;
mod logs;
mod db;

use teloxide::prelude::*;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

use crate::db::RedisConnection;
use crate::dialogue::Dialogue;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env().auto_send();

    // teloxide::dialogues_repl(bot, |message, dialogue| async move {
    //     handle_message(message, dialogue)
    //         .await
    //         .expect("Some problem happened")
    // })
    // .await;
    
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(args);

    let db_shared: Arc<Mutex<RedisConnection>> = Arc::new(Mutex::new(
        match db::RedisConnection::new(&config.redis_ip[..]).await {
            Ok(v) => v,
            Err(err) => panic!("Could not start redis connection: {}", err),
        }
    ));

    Dispatcher::new(bot)
        .messages_handler(|rx: UnboundedReceiver<UpdateWithCx<AutoSend<Bot>, Message>>| async move {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, |cx | async {
                let new_db_handle = Arc::clone(&db_shared);
                handle_message(cx, new_db_handle).await
            }).await;
        }
        ).dispatch().await;
        log::info!("Closing the bot...");
}

struct Config {
    redis_ip: String
}

fn parse_args(args: Vec<String>) -> Config {
    match args.len() {
        1 => {
            Config{ redis_ip: String::from("redis://127.0.0.1/") }
        }
        2 => {
            Config{ redis_ip: String::from("redis://") + &args[1][..] + "/" }
        }
        _ => {
            print_usage();
            panic!();
        }
    }
}

fn print_usage() {
    println!("Telegram bot. Run with no arguments or specify redis ip as first argument (without 'redis://' prefix).")
}

async fn handle_dialogue(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    use teloxide::types::{MediaKind, MessageKind};
    use crate::dialogue::Answer;

    // Don't know hot to avoid repeating of this code properly
    fn default_response(
        cx: UpdateWithCx<AutoSend<Bot>, Message>,
        dialogue: Dialogue,
    ) -> TransitionOut<Dialogue> {
        log::info!(
            "{}",
            logs::format_log_chat("Received something else", cx.chat_id())
        );
        cx.answer("Send a sticker to start.");
        next(dialogue)
    }

    match &cx.update.kind {
        MessageKind::Common(cmn) => {
            let ans: Answer;
            match &cmn.media_kind {
                MediaKind::Text(media) => {
                    log::info!("{}", logs::format_log_chat("Received a text", cx.chat_id()));
                    ans = Answer::String(media.text.clone());
                }
                MediaKind::Sticker(media) => {
                    log::info!(
                        "{}",
                        logs::format_log_chat("Received a sticker", cx.chat_id())
                    );
                    ans = Answer::Sticker(media.sticker.clone());
                }
                _ => {
                    return default_response(cx, dialogue);
                }
            }
            let res = dialogue.react(cx, ans).await;
            res
        }
        _ => default_response(cx, dialogue),
    }
}

async fn handle_message(cx: UpdateWithCx<AutoSend<Bot>, Message>, db_shared: Arc<Mutex<RedisConnection>>) {

    
    let mut db_con = db_shared.lock().await;

    let chat_id = cx.update.chat_id();
    let from_id = cx.update.from().map(|u| u.id);
    let dialogue: Dialogue = match db_con.get_dialogue(chat_id, from_id).await.map(Option::unwrap_or_default) {
        Ok(d) => d,
        Err(e) => {
            log::info!(
                "{}",
                logs::format_log_chat(&format!("Could not get dialogue (from {f:?}): {e:?}", f=from_id, e=e), chat_id)
            );
            return
        },
    };

    let stage = match handle_dialogue(cx, dialogue).await {
        Ok(a) => a,
        Err(e) => {
            log::info!(
                "{}",
                logs::format_log_chat(&format!("Could not handle dialogue (from {f:?}): {e:?}", f=from_id, e=e), chat_id)
            );
            return
        },
    };

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