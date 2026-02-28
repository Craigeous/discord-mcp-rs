use std::sync::Arc;
use twilight_http::Client;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- get_current_user --

pub async fn get_current_user(discord: &Arc<Client>) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.current_user().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(user) => json_result(&user),
        Err(e) => deserialize_error(e),
    }
}

// -- list_guilds --

pub async fn list_guilds(discord: &Arc<Client>) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.current_user_guilds().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(guilds) => json_result(&guilds),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild(
    discord: &Arc<Client>,
    params: GetGuildParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(guild) => json_result(&guild),
        Err(e) => deserialize_error(e),
    }
}
