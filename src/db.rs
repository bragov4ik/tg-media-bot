use redis::AsyncCommands;
use redis::RedisResult;
use crate::logs::format_log_chat;
use log::info;
/*
Wanted to use trait DBConnection with methods set_alias and get_sticker_id
but async functions in traits are not that trivial, might add it later.
 */

pub struct RedisConnection {
    connection: redis::aio::Connection ,
}

impl RedisConnection {
    async fn new(redis_ip: &str) -> redis::RedisResult<RedisConnection> {
        let client = redis::Client::open(redis_ip)?;
        let con = client.get_async_connection().await?;
        Ok(RedisConnection{ connection: con })
    }
}

// Database connection
impl RedisConnection {
    fn get_chat_key(chat_id: i64) -> String {
        format!("chat:{}", chat_id)
    }

    async fn set_alias(&mut self, chat_id: i64, sticker_id: &str, alias: &str) {
        let chat_key: String = RedisConnection::get_chat_key(chat_id);
        let set_result: RedisResult<()> = self.connection.hset(chat_key, alias, sticker_id).await;
        match set_result {
            Ok(_) => {
                info!("{}", format_log_chat(&format!("Saved alias '{a}' for '{s}'", a=alias, s=sticker_id), chat_id));
            }
            Err(e) => {
                info!("{}", format_log_chat(&format!("Failed to save alias to DB: {}", e), chat_id));
            }
        }
    }

    async fn set_aliases<'a, T>(&mut self, chat_id: i64, sticker_id: &str, aliases: T) 
        where
            T: IntoIterator<Item = &'a str> {
        for alias in aliases {
            self.set_alias(chat_id, sticker_id, alias).await;
        }
    }

    async fn get_sticker_id(&mut self, chat_id: i64, alias: &str) -> Option<String> {
        let chat_key: String = RedisConnection::get_chat_key(chat_id);
        let set_result: RedisResult<String> = self.connection.hget(chat_key, alias).await;
        match set_result {
            Ok(sticker_id) => {
                info!("{}", format_log_chat(&format!("Retrieved '{s}' by alias '{a}'", a=alias, s=sticker_id), chat_id));
                Some(sticker_id)
            }
            Err(e) => {
                info!("{}", format_log_chat(&format!("Failed find alias '{a}' in DB: {}", e, a=alias), chat_id));
                None
            }
        }
    }
}