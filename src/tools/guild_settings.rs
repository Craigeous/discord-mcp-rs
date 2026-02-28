use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- update_guild --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGuildParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// New guild name
    pub name: Option<String>,
    /// New guild description
    pub description: Option<String>,
}

pub async fn update_guild(
    discord: &Arc<Client>,
    params: UpdateGuildParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.update_guild(guild_id);

    if let Some(ref name) = params.name {
        req = req.name(name.as_str());
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(guild) => json_result(&guild),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_prune_count --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildPruneCountParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Number of days of inactivity (1-30, default 7)
    pub days: Option<u16>,
}

pub async fn get_guild_prune_count(
    discord: &Arc<Client>,
    params: GetGuildPruneCountParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.guild_prune_count(guild_id);

    if let Some(days) = params.days {
        req = req.days(days);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(count) => json_result(&count),
        Err(e) => deserialize_error(e),
    }
}

// -- begin_guild_prune --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BeginGuildPruneParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Number of days of inactivity (1-30, default 7)
    pub days: Option<u16>,
    /// Whether to return the number of pruned members (may time out for large guilds)
    pub compute_prune_count: Option<bool>,
}

pub async fn begin_guild_prune(
    discord: &Arc<Client>,
    params: BeginGuildPruneParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.create_guild_prune(guild_id);

    if let Some(days) = params.days {
        req = req.days(days);
    }
    if let Some(compute) = params.compute_prune_count {
        req = req.compute_prune_count(compute);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(result) => json_result(&result),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_vanity_url --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildVanityUrlParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild_vanity_url(
    discord: &Arc<Client>,
    params: GetGuildVanityUrlParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_vanity_url(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(vanity) => json_result(&vanity),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_welcome_screen --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildWelcomeScreenParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild_welcome_screen(
    discord: &Arc<Client>,
    params: GetGuildWelcomeScreenParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_welcome_screen(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(screen) => json_result(&screen),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_widget --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildWidgetParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild_widget(
    discord: &Arc<Client>,
    params: GetGuildWidgetParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_widget(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(widget) => json_result(&widget),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_voice_regions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildVoiceRegionsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild_voice_regions(
    discord: &Arc<Client>,
    params: GetGuildVoiceRegionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_voice_regions(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(regions) => json_result(&regions),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_preview --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildPreviewParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn get_guild_preview(
    discord: &Arc<Client>,
    params: GetGuildPreviewParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_preview(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(preview) => json_result(&preview),
        Err(e) => deserialize_error(e),
    }
}
