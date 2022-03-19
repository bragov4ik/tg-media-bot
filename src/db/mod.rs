//! Database connection.
//!
//! Handles and provides an interface to the database for bot.

mod redis;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

#[async_trait]
pub trait StickerStorage {
    type Error;

    async fn set_alias(
        &mut self, chat_id: i64, alias: &str, sticker_id: &str
    ) -> Result<(), Self::Error>;
    async fn get_sticker_id(
        &mut self, chat_id: i64, alias: &str
    ) -> Result<Option<String>, Self::Error>;
    async fn remove_alias(&mut self, chat_id: i64, alias: &str) -> Result<(), Self::Error>;
    async fn get_aliases(
        &mut self, chat_id: i64
    ) -> Result<Option<HashMap<String, Vec<String>>>, Self::Error>;
}

#[async_trait]
pub trait DialogueStorage {
    type Error;

    async fn update_dialogue<'a, D>(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
        dialogue: D,
    ) -> Result<(), Self::Error>
    where
        D: Serialize;

    async fn get_dialogue<'a, D>(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
    ) -> Result<Option<D>, Self::Error>
    where
        D: DeserializeOwned;

    async fn remove_dialogue(
        &mut self,
        chat_id: i64,
        from_id: Option<i64>,
    ) -> Result<(), Self::Error>;
}