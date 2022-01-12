//! Database connection.
//!
//! Handles and provides an interface to the database for bot.

use crate::utils::format_log_chat;
use redis::AsyncCommands;
use redis::RedisResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

/* TODO: wrap connection in trait (not trivial with async). */
/// Redis connection representation.
///
/// Provides simple interface for storing sticker aliases and dialogue
/// state (with serialization).
pub struct RedisConnection {
    connection: redis::aio::Connection,
}

// General implementation
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
}

impl RedisConnection {
    /// Get redis key for aliases storage.
    fn get_aliases_key(chat_id: i64) -> String {
        RedisConnection::get_chat_key(chat_id) + "aliases"
    }

    /// Store alias-sticker mapping in redis.
    ///
    /// If the alias is already tied to some sticker, overwrite it
    /// so the alias will be mapped to a new sticker (for given
    /// `chat_id`). `sticker_id` is a file ID of the sticker.
    pub async fn set_alias(&mut self, chat_id: i64, alias: &str, sticker_id: &str) {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let set_result: RedisResult<()> = self.connection.hset(key, alias, sticker_id).await;
        match set_result {
            Ok(_) => {
                log::info!(
                    "{}",
                    format_log_chat(
                        &format!("Saved alias '{a}' for '{s}'", a = alias, s = sticker_id),
                        chat_id
                    )
                );
            }
            Err(e) => {
                log::info!(
                    "{}",
                    format_log_chat(&format!("Failed to save alias to DB: {}", e), chat_id)
                );
            }
        }
    }

    /// Obtain sticker file id for given alias in the chat (if any).
    pub async fn get_sticker_id(&mut self, chat_id: i64, alias: &str) -> Option<String> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let set_result: RedisResult<String> = self.connection.hget(key, alias).await;
        match set_result {
            Ok(sticker_id) => {
                log::info!(
                    "{}",
                    format_log_chat(
                        &format!("Retrieved '{s}' by alias '{a}'", a = alias, s = sticker_id),
                        chat_id
                    )
                );
                Some(sticker_id)
            }
            Err(e) => {
                log::info!(
                    "{}",
                    format_log_chat(
                        &format!("Failed find alias '{a}' in DB: {}", e, a = alias),
                        chat_id
                    )
                );
                None
            }
        }
    }

    /// Unmap (remove) the alias for given chat id.
    pub async fn remove_alias(
        &mut self,
        chat_id: i64,
        alias: &str,
    ) -> Result<(), RedisStorageError> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        let n_removed: i64 = self.connection.hdel(key, alias).await.map_err(|e| {
            log::info!(
                "{}",
                format_log_chat(&format!("Failed to remove alias from DB: {}", e), chat_id)
            );
            RedisStorageError::RedisError(e)
        })?;
        // Log and form result
        match n_removed {
            0 => {
                log::info!(
                    "{}",
                    format_log_chat(&format!("Alias '{a}' was not found", a = alias), chat_id)
                );
                Err(RedisStorageError::AliasNotFound)
            }
            1 => {
                log::info!(
                    "{}",
                    format_log_chat(&format!("Removed alias '{a}'", a = alias), chat_id)
                );
                Ok(())
            }
            n_unexpected => {
                log::warn!(
                    "{}",
                    format_log_chat(
                        &format!(
                            "'{a}' removal returned unexpected number: {n}",
                            a = alias,
                            n = n_unexpected
                        ),
                        chat_id
                    )
                );
                Ok(())
            }
        }
    }

    /// Get mapping of all stickers to aliases in the chat.
    /// Intended for listing the aliases.
    pub async fn get_aliases(&mut self, chat_id: i64) -> Option<HashMap<String, Vec<String>>> {
        let key: String = RedisConnection::get_aliases_key(chat_id);
        // TODO: replace with hscan to avoid blocking DB.
        // this way should work for now, at small scale.
        let get_result: RedisResult<Vec<String>> = self.connection.hgetall(key).await;
        if let Ok(list_result) = get_result {
            let mut mapping: HashMap<String, Vec<String>> = HashMap::new();
            for pair in list_result.chunks(2) {
                if let [alias, sticker_id] = pair {
                    match mapping.get_mut(sticker_id) {
                        Some(list) => {
                            log::trace!("Retrieved list {:#?} from mapping", list);
                            list.push(alias.to_string());
                        }
                        None => {
                            log::trace!(
                                "No list for sticker w/ alias {} was found, creating",
                                alias
                            );
                            mapping.insert(sticker_id.to_string(), vec![alias.to_string()]);
                        }
                    }
                } else {
                    log::error!(
                        "{}",
                        format_log_chat(
                            &format!(
                                "Invalid result of `HGETALL`: pair {p:#?}\
                        does not match key-value pattern",
                                p = pair
                            ),
                            chat_id
                        )
                    )
                }
            }
            Some(mapping)
        } else {
            None
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
            .unwrap_or("NO_ID".to_string())
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
        let value: String = serde_json::to_string(&dialogue).map_err(|err| {
            log::info!(
                "{}",
                format_log_chat(&format!("Failed to serialize dialogue: {}", err), chat_id)
            );
            RedisStorageError::SerdeError(err)
        })?;

        // Save
        let set_result: RedisResult<()> = self.connection.hset(&key, &field, &value).await;
        match &set_result {
            Ok(_) => {
                log::info!(
                    "{}",
                    format_log_chat(&format!("Saved dialogue for '{f}'", f = field), chat_id)
                );
            }
            Err(err) => {
                log::info!(
                    "{}",
                    format_log_chat(&format!("Failed to save dialogue to DB: {}", err), chat_id)
                );
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
        let value: Option<String> = self
            .connection
            .hget(&key, &field)
            .await
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
