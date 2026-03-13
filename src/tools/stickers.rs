use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_guild_stickers --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildStickersParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_stickers(
    discord: &Arc<Client>,
    params: ListGuildStickersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_stickers(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(stickers) => json_result(&stickers),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_sticker --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildStickerParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The sticker ID
    pub sticker_id: String,
}

pub async fn get_guild_sticker(
    discord: &Arc<Client>,
    params: GetGuildStickerParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let sticker_id = parse_id(&params.sticker_id)?;
    let response = match discord.guild_sticker(guild_id, sticker_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(sticker) => json_result(&sticker),
        Err(e) => deserialize_error(e),
    }
}

// -- create_guild_sticker --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGuildStickerParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Sticker name (2-30 characters)
    pub name: String,
    /// Sticker description (2-100 characters)
    pub description: String,
    /// Comma-separated autocomplete tags for the sticker (max 200 characters)
    pub tags: String,
    /// Path to the sticker image file (png, apng, gif, or json for Lottie)
    pub file_path: String,
}

pub async fn create_guild_sticker(
    discord: &Arc<Client>,
    params: CreateGuildStickerParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let file_bytes = std::fs::read(&params.file_path).map_err(|e| {
        rmcp::ErrorData::invalid_params(format!("Failed to read file: {e}"), None)
    })?;

    let response = match discord
        .create_guild_sticker(guild_id, &params.name, &params.description, &params.tags, &file_bytes)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(sticker) => json_result(&sticker),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_guild_sticker --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteGuildStickerParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The sticker ID
    pub sticker_id: String,
}

pub async fn delete_guild_sticker(
    discord: &Arc<Client>,
    params: DeleteGuildStickerParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let sticker_id = parse_id(&params.sticker_id)?;
    match discord.delete_guild_sticker(guild_id, sticker_id).await {
        Ok(_) => text_result("Sticker deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- get_sticker --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetStickerParams {
    /// The sticker ID
    pub sticker_id: String,
}

pub async fn get_sticker(
    discord: &Arc<Client>,
    params: GetStickerParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let sticker_id = parse_id(&params.sticker_id)?;
    let response = match discord.sticker(sticker_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(sticker) => json_result(&sticker),
        Err(e) => deserialize_error(e),
    }
}

// -- list_sticker_packs --

pub async fn list_sticker_packs(
    discord: &Arc<Client>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.nitro_sticker_packs().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    // StickerPackListing doesn't implement Serialize, so use the raw bytes
    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => return deserialize_error(e),
    };
    let json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
    })?;
    json_result(&json)
}
