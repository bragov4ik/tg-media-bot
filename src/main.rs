mod dialogue;
mod logs;
mod db;

use teloxide::prelude::*;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::sync::Arc;
// TODO: get rid of using tokio's Mutex https://tokio.rs/tokio/tutorial/channels
use tokio::sync::Mutex;

use crate::db::RedisConnection;

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
                handle_message(cx.update, new_db_handle).await
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

async fn handle_message(msg: Message, db: Arc<Mutex<RedisConnection>>) {
    use teloxide::types::{MediaKind, MessageKind};
    use crate::dialogue::{Answer, Dialogue};

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
    


    match msg.kind {
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
                default_response(cx, dialogue);
            }
        }
        let res = dialogue.react(cx, ans).await;
        res
        }
        _ => default_response(cx, dialogue),
    }
}

// async fn handle_message(
//     cx: UpdateWithCx<AutoSend<Bot>, Message>,
//     dialogue: Dialogue,
// ) -> TransitionOut<Dialogue> {

//     match &cx.update.kind {
//         MessageKind::Common(cmn) => {
//             let ans: Answer;
//             match &cmn.media_kind {
//                 MediaKind::Text(media) => {
//                     log::info!("{}", logs::format_log_chat("Received a text", cx.chat_id()));
//                     ans = Answer::String(media.text.clone());
//                 }
//                 MediaKind::Sticker(media) => {
//                     log::info!(
//                         "{}",
//                         logs::format_log_chat("Received a sticker", cx.chat_id())
//                     );
//                     ans = Answer::Sticker(media.sticker.clone());
//                 }
//                 _ => {
//                     return default_response(cx, dialogue);
//                 }
//             }
//             let res = dialogue.react(cx, ans).await;
//             res
//         }
//         _ => default_response(cx, dialogue),
//     }
// }
