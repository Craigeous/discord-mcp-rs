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
