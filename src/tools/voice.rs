use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- list_voice_regions --

pub async fn list_voice_regions(
    discord: &Arc<Client>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.voice_regions().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(regions) => json_result(&regions),
        Err(e) => deserialize_error(e),
    }
}

// -- update_current_user_voice_state --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateCurrentUserVoiceStateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The stage channel ID the user is currently connected to
    pub channel_id: Option<String>,
    /// Whether to suppress the user (set to true to mute in stage). Calling this toggles suppress on.
    pub suppress: Option<bool>,
    /// ISO8601 timestamp for when the user requested to speak (empty string to remove)
    pub request_to_speak_timestamp: Option<String>,
}

pub async fn update_current_user_voice_state(
    discord: &Arc<Client>,
    params: UpdateCurrentUserVoiceStateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.update_current_user_voice_state(guild_id);

    if let Some(ref cid) = params.channel_id {
        req = req.channel_id(parse_id(cid)?);
    }
    if params.suppress.unwrap_or(false) {
        req = req.suppress();
    }
    if let Some(ref ts) = params.request_to_speak_timestamp {
        req = req.request_to_speak_timestamp(ts);
    }

    match req.await {
        Ok(_) => crate::error::text_result("Voice state updated"),
        Err(e) => discord_api_error(e),
    }
}

// -- update_user_voice_state --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateUserVoiceStateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID
    pub user_id: String,
    /// The stage channel ID the user is currently connected to
    pub channel_id: String,
    /// Whether to suppress the user (set to true to mute in stage)
    pub suppress: Option<bool>,
}

pub async fn update_user_voice_state(
    discord: &Arc<Client>,
    params: UpdateUserVoiceStateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    let channel_id = parse_id(&params.channel_id)?;

    let mut req = discord.update_user_voice_state(guild_id, user_id, channel_id);

    if params.suppress.unwrap_or(false) {
        req = req.suppress();
    }

    match req.await {
        Ok(_) => crate::error::text_result("User voice state updated"),
        Err(e) => discord_api_error(e),
    }
}
