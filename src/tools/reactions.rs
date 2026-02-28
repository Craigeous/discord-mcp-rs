use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::EmojiMarker;
use twilight_http::request::channel::reaction::RequestReactionType;

use crate::error::{discord_api_error, json_result, text_result};
use crate::util::parse_id;

/// Parse an emoji string into a RequestReactionType.
/// Supports: unicode chars ("👍"), or custom emoji "name:id" format.
fn parse_emoji(emoji_str: &str) -> Result<RequestReactionType<'_>, rmcp::ErrorData> {
    if let Some((name, id_str)) = emoji_str.rsplit_once(':') {
        if let Ok(id) = id_str.parse::<u64>() {
            if id != 0 {
                return Ok(RequestReactionType::Custom {
                    id: Id::<EmojiMarker>::new(id),
                    name: Some(name),
                });
            }
        }
    }
    Ok(RequestReactionType::Unicode { name: emoji_str })
}

// -- add_reaction --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddReactionParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
    /// Emoji: unicode character (e.g. "👍") or custom emoji "name:id"
    pub emoji: String,
}

pub async fn add_reaction(
    discord: &Arc<Client>,
    params: AddReactionParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let emoji = parse_emoji(&params.emoji)?;
    match discord.create_reaction(channel_id, message_id, &emoji).await {
        Ok(_) => text_result("Reaction added"),
        Err(e) => discord_api_error(e),
    }
}

// -- remove_reaction --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveReactionParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
    /// Emoji: unicode character or custom "name:id"
    pub emoji: String,
    /// User ID to remove reaction for (omit to remove bot's own reaction)
    pub user_id: Option<String>,
}

pub async fn remove_reaction(
    discord: &Arc<Client>,
    params: RemoveReactionParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let emoji = parse_emoji(&params.emoji)?;

    let result = if let Some(ref uid) = params.user_id {
        let user_id = parse_id(uid)?;
        discord
            .delete_reaction(channel_id, message_id, &emoji, user_id)
            .await
    } else {
        discord
            .delete_current_user_reaction(channel_id, message_id, &emoji)
            .await
    };
    match result {
        Ok(_) => text_result("Reaction removed"),
        Err(e) => discord_api_error(e),
    }
}

// -- get_reactions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetReactionsParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
    /// Emoji: unicode character or custom "name:id"
    pub emoji: String,
    /// Max number of users to return (1-100, default 25)
    pub limit: Option<u16>,
}

pub async fn get_reactions(
    discord: &Arc<Client>,
    params: GetReactionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let emoji = parse_emoji(&params.emoji)?;

    let mut req = discord.reactions(channel_id, message_id, &emoji);
    if let Some(limit) = params.limit {
        req = req.limit(limit);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(users) => json_result(&users),
        Err(e) => crate::error::deserialize_error(e),
    }
}

// -- clear_all_reactions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClearAllReactionsParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
}

pub async fn clear_all_reactions(
    discord: &Arc<Client>,
    params: ClearAllReactionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    match discord.delete_all_reactions(channel_id, message_id).await {
        Ok(_) => text_result("All reactions cleared"),
        Err(e) => discord_api_error(e),
    }
}

// -- clear_emoji_reactions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClearEmojiReactionsParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
    /// Emoji to clear: unicode character or custom "name:id"
    pub emoji: String,
}

pub async fn clear_emoji_reactions(
    discord: &Arc<Client>,
    params: ClearEmojiReactionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let emoji = parse_emoji(&params.emoji)?;
    match discord
        .delete_all_reaction(channel_id, message_id, &emoji)
        .await
    {
        Ok(_) => text_result("Emoji reactions cleared"),
        Err(e) => discord_api_error(e),
    }
}
