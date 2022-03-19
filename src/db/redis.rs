//! Redis connection.

use crate::utils::{ log_chat, log_time };
use super::StickerStorage;
use async_trait::async_trait;
use redis::AsyncCommands;
use redis::RedisResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;


/* TODO: wrap connection in trait (not trivial with async). */
/// Redis connection representation.
///
/// Provides simple interface for storing sticker aliases and dialogue
/// state (with serialization).
pub struct RedisConnection {
    connection: redis::aio::Connection,
}

impl RedisConnection {
    /// Create new connection to redis server in specified ip.
    ///
    /// IP should be formatted according to `redis` crate requirements
    /// (currently similar to `redis://127.0.0.1/`)
    pub async fn new(redis_ip: &str) -> redis::RedisResult<RedisConnection> {
        let client = redis::Client::open(redis_ip)?;
        let con = client.get_async_connection().await?;
        Ok(RedisConnection { connection: con })
    }

    /// Get redis key for chat given its identifier.
    fn get_chat_key(chat_id: i64) -> String {
        format!("chat:{}", chat_id)
    }

    /// Get redis key for aliases storage.
    fn get_aliases_key(chat_id: i64) -> String {
        RedisConnection::get_chat_key(chat_id) + "aliases"
    }
}

#[async_trait]
impl StickerStorage for RedisConnection {
    type Error = RedisStorageError;
    /// Store alias-sticker mapping in redis.
    ///
    /// If the alias is already tied to some sticker, overwrite it
    /// so the alias will be mapped to a new sticker (for given
    /// `chat_id`). `sticker_id` is a file ID of the sticker.
    async fn set_alias(
        &mut self, chat_id: i64, alias: &str, sticker_id: &str
    ) -> Result<(), Self::Error> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let set_result: Result<(), Self::Error> =
            self.connection.hset(
                key, alias, sticker_id
            ).await
                .map_err(RedisStorageError::RedisError);

        // Log
        match &set_result {
            Ok(_) => log_chat!(
                log::Level::Info, chat_id,
                "Saved alias '{a}' for '{s}'", a = alias, s = sticker_id
            ),
            Err(e) => log_chat!(
                log::Level::Info, chat_id,
                "Failed to save alias to DB: {}", e
            ),
        }
        set_result
    }

    /// Obtain sticker file id for given alias in the chat (if any).
    async fn get_sticker_id(
        &mut self, chat_id: i64, alias: &str
    ) -> Result<Option<String>, Self::Error> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let set_result: Result<Option<String>, Self::Error> = 
            self.connection.hget(key, alias).await
                .map(Some)
                .map_err(RedisStorageError::RedisError);

        // Log
        match &set_result {
            Ok(Some(sticker_id)) => {
                log_chat!(
                    log::Level::Info, chat_id, "Retrieved '{s}' by alias '{a}'",
                    a = alias, s = sticker_id
                );
            }
            Ok(None) => {
                unreachable!() // because we always assign Some in map
            }
            Err(e) => {
                log_chat!(
                    log::Level::Info, chat_id,
                    "Failed to retrieve alias '{a}' in DB: {}", e, a = alias
                );
            }
        };
        set_result
    }

    /// Unmap (remove) the alias for given chat id.
    async fn remove_alias(
        &mut self, chat_id: i64, alias: &str
    ) -> Result<(), Self::Error> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let n_removed: i64 = self.connection.hdel(key, alias).await.map_err(|e| {
            log_chat!(log::Level::Info, chat_id, "Failed to remove alias from DB: {}", e);
            RedisStorageError::RedisError(e)
        })?;
        // Log and form result
        match n_removed {
            0 => {
                log_chat!(log::Level::Info, chat_id, "Alias '{a}' was not found", a = alias);
                Err(RedisStorageError::AliasNotFound)
            }
            1 => {
                log_chat!(log::Level::Info, chat_id, "Removed alias '{a}'", a = alias);
                Ok(())
            }
            n_unexpected => {
                log_chat!(
                    log::Level::Warn, chat_id,
                    "'{a}' removal returned unexpected number: {n}",
                    a = alias,
                    n = n_unexpected
                );
                Ok(())
            }
        }
    }

    /// Get mapping of all stickers to aliases in the chat.
    /// Intended for listing the aliases.
    async fn get_aliases(
        &mut self, chat_id: i64
    ) -> Result<Option<HashMap<String, Vec<String>>>, Self::Error> {
        fn pairs_to_mapping(pairs_list: Vec<(&String, &String)>) -> HashMap<String, Vec<String>> {
            let mut mapping: HashMap<String, Vec<String>> = HashMap::new();
            for (alias, sticker_id) in pairs_list {
                match mapping.get_mut(sticker_id) {
                    Some(list) => {
                        log_time!(log::Level::Trace, "Retrieved list {:#?} from mapping", list);
                        list.push(alias.to_string());
                    }
                    None => {
                        log_time!(
                            log::Level::Trace,
                            "No list for sticker w/ alias {} was found, creating", alias
                        );
                        mapping.insert(sticker_id.to_string(), vec![alias.to_string()]);
                    }
                }
                    // 
            }
            mapping
        }

        let key: String = RedisConnection::get_aliases_key(chat_id);
        // TODO: replace with hscan to avoid blocking DB.
        // this way should work for now, at small scale.
        let get_result: Result<Vec<String>, Self::Error> = self.connection.hgetall(key).await
            .map_err(RedisStorageError::RedisError);
        match get_result {
            Ok(list_result) => {
                let pairs_list: Vec<(&String, &String)> = list_result.chunks(2)
                    .filter_map(|c| {
                        if let [a, b] = c {
                            Some((a, b))
                        } else {
                            log_chat!(
                                log::Level::Warn, chat_id,
                                "Invalid result of `HGETALL`: pair {:#?} does not match \
                                key-value pattern", c
                            );
                            None
                        }
                    })
                    .collect();
                let mapping = pairs_to_mapping(pairs_list);                
                Ok(Some(mapping))
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

/// An error returned from `Storage` implementation.
#[derive(Debug)]
pub enum RedisStorageError {
    SerdeError(serde_json::Error),

    RedisError(redis::RedisError),

    /// Returned from [`remove_dialogue`].
    DialogueNotFound,

    /// Returned from [`remove_alias`]
    AliasNotFound,
}

impl Display for RedisStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisStorageError::SerdeError(e) => e.fmt(f),
            RedisStorageError::RedisError(e) => e.fmt(f),
            RedisStorageError::DialogueNotFound => f.write_str("DialogueNotFound"),
            RedisStorageError::AliasNotFound => f.write_str("AliasNotFound"),
        }
    }
}

/// Dialogue storage.
///
/// Similar to `teloxide::dispatching::dialogue::Storage`,
/// but with different dialogue for each user in the chat.
impl RedisConnection {
    /// Get redis key for dialogues storage for given chat id.
    fn get_dialogues_key(chat_id: i64) -> String {
        RedisConnection::get_chat_key(chat_id) + "dialogues"
    }

    /// Get field name for given from_id (can be empty).
    fn get_from_field(from_id: Option<i64>) -> String {
        from_id
            .map(|x| x.to_string())
            .unwrap_or_else(|| "NO_ID".to_owned())
    }

    /// Update a dialogue in the storage.
    ///
    /// Saves the `dialogue` in the redis database for given chat and user.
    pub async fn update_dialogue<'a, D>(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
        dialogue: D,
    ) -> Result<(), RedisStorageError>
    where
        D: Serialize,
    {
        let key: String = RedisConnection::get_dialogues_key(chat_id);
        let field: String = RedisConnection::get_from_field(from_id);

        // Serialize
        let value: String = serde_json::to_string(&dialogue)
            .map_err(|err| {
                RedisStorageError::SerdeError(err)
            })?;

        // Save
        let set_result: RedisResult<()> = self.connection.hset(&key, &field, &value).await;
        match &set_result {
            Ok(_) => {
                log_chat!(log::Level::Info, chat_id, "Saved dialogue for '{f}'", f = field);
            }
            Err(err) => {
                log_chat!(log::Level::Info, chat_id, "Failed to save dialogue to DB: {}", err);
            }
        }
        set_result.map_err(RedisStorageError::RedisError)
    }

    /// Retrieve a dialogue from the storage.
    ///
    /// Give the `dialogue` for given chat and user.
    pub async fn get_dialogue<'a, D>(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
    ) -> Result<Option<D>, RedisStorageError>
    where
        D: DeserializeOwned,
    {
        let key: String = RedisConnection::get_dialogues_key(chat_id);
        let field: String = RedisConnection::get_from_field(from_id);

        // Retrieve from DB
        let value: Option<String> = self.connection
            .hget(&key, &field).await
            .map_err(RedisStorageError::RedisError)?;

        // Deserialize
        let value: Result<Option<D>, RedisStorageError> = value
            .map(|v| serde_json::from_str::<D>(&v[..]))
            .transpose()
            .map_err(RedisStorageError::SerdeError);
        value
    }

    /// Remove dialogue.
    pub async fn remove_dialogue(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
    ) -> Result<(), RedisStorageError> {
        let key: String = RedisConnection::get_dialogues_key(chat_id);
        let field: String = RedisConnection::get_from_field(from_id);

        let del_res: RedisResult<i64> = self.connection.hdel(key, field).await;
        match del_res {
            Ok(0) => Err(RedisStorageError::DialogueNotFound),
            Ok(_) => Ok(()),
            Err(e) => Err(RedisStorageError::RedisError(e)),
        }
    }
}
