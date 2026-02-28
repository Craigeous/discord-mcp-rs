use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_scheduled_events --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListScheduledEventsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_scheduled_events(
    discord: &Arc<Client>,
    params: ListScheduledEventsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_scheduled_events(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(events) => json_result(&events),
        Err(e) => deserialize_error(e),
    }
}

// -- get_scheduled_event --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetScheduledEventParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The scheduled event ID
    pub event_id: String,
}

pub async fn get_scheduled_event(
    discord: &Arc<Client>,
    params: GetScheduledEventParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let event_id = parse_id(&params.event_id)?;
    let response = match discord.guild_scheduled_event(guild_id, event_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(event) => json_result(&event),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_scheduled_event --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteScheduledEventParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The scheduled event ID
    pub event_id: String,
}

pub async fn delete_scheduled_event(
    discord: &Arc<Client>,
    params: DeleteScheduledEventParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let event_id = parse_id(&params.event_id)?;
    match discord.delete_guild_scheduled_event(guild_id, event_id).await {
        Ok(_) => text_result("Scheduled event deleted"),
        Err(e) => discord_api_error(e),
    }
}

// -- list_scheduled_event_users --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListScheduledEventUsersParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The scheduled event ID
    pub event_id: String,
    /// Max number of users to return (default 100)
    pub limit: Option<u16>,
}

pub async fn list_scheduled_event_users(
    discord: &Arc<Client>,
    params: ListScheduledEventUsersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let event_id = parse_id(&params.event_id)?;

    let mut req = discord.guild_scheduled_event_users(guild_id, event_id);

    if let Some(limit) = params.limit {
        req = req.limit(limit);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(users) => json_result(&users),
        Err(e) => deserialize_error(e),
    }
}
