use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_emojis --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListEmojisParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_emojis(
    discord: &Arc<Client>,
    params: ListEmojisParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.emojis(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(emojis) => json_result(&emojis),
        Err(e) => deserialize_error(e),
    }
}

// -- get_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetEmojiParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The emoji ID
    pub emoji_id: String,
}

pub async fn get_emoji(
    discord: &Arc<Client>,
    params: GetEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let emoji_id = parse_id(&params.emoji_id)?;
    let response = match discord.emoji(guild_id, emoji_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emoji) => json_result(&emoji),
        Err(e) => deserialize_error(e),
    }
}

// -- create_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateEmojiParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Emoji name
    pub name: String,
    /// Base64-encoded image data (data URI format: data:image/png;base64,...)
    pub image: String,
}

pub async fn create_emoji(
    discord: &Arc<Client>,
    params: CreateEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord
        .create_emoji(guild_id, &params.name, &params.image)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emoji) => json_result(&emoji),
        Err(e) => deserialize_error(e),
    }
}

// -- update_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateEmojiParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The emoji ID
    pub emoji_id: String,
    /// New emoji name
    pub name: Option<String>,
}

pub async fn update_emoji(
    discord: &Arc<Client>,
    params: UpdateEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let emoji_id = parse_id(&params.emoji_id)?;

    let mut req = discord.update_emoji(guild_id, emoji_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emoji) => json_result(&emoji),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteEmojiParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The emoji ID
    pub emoji_id: String,
}

pub async fn delete_emoji(
    discord: &Arc<Client>,
    params: DeleteEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let emoji_id = parse_id(&params.emoji_id)?;
    match discord.delete_emoji(guild_id, emoji_id).await {
        Ok(_) => text_result("Emoji deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// ========================
// APPLICATION EMOJIS
// ========================

// -- list_application_emojis --

pub async fn list_application_emojis(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.get_application_emojis(application_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emojis) => json_result(&emojis),
        Err(e) => deserialize_error(e),
    }
}

// -- create_application_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateApplicationEmojiParams {
    /// Emoji name
    pub name: String,
    /// Base64-encoded image data (data URI format: data:image/png;base64,...)
    pub image: String,
}

pub async fn create_application_emoji(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: CreateApplicationEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord
        .add_application_emoji(application_id, &params.name, &params.image)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emoji) => json_result(&emoji),
        Err(e) => deserialize_error(e),
    }
}

// -- update_application_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateApplicationEmojiParams {
    /// The emoji ID
    pub emoji_id: String,
    /// New emoji name
    pub name: String,
}

pub async fn update_application_emoji(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: UpdateApplicationEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let emoji_id = parse_id(&params.emoji_id)?;
    let response = match discord
        .update_application_emoji(application_id, emoji_id, &params.name)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(emoji) => json_result(&emoji),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_application_emoji --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteApplicationEmojiParams {
    /// The emoji ID
    pub emoji_id: String,
}

pub async fn delete_application_emoji(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: DeleteApplicationEmojiParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let emoji_id = parse_id(&params.emoji_id)?;
    match discord
        .delete_application_emoji(application_id, emoji_id)
        .await
    {
        Ok(_) => text_result("Application emoji deleted"),
        Err(e) => discord_api_error(e),
    }
}
