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

// -- create_scheduled_event --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateScheduledEventParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Event name
    pub name: String,
    /// Privacy level: 2=guild_only
    pub privacy_level: u8,
    /// Scheduled start time (ISO8601 timestamp)
    pub scheduled_start_time: String,
    /// Scheduled end time (ISO8601 timestamp, required for external events)
    pub scheduled_end_time: Option<String>,
    /// Entity type: 1=stage_instance, 2=voice, 3=external
    pub entity_type: u8,
    /// Channel ID (required for stage_instance and voice types)
    pub channel_id: Option<String>,
    /// Location (required for external entity type)
    pub location: Option<String>,
    /// Event description
    pub description: Option<String>,
}

pub async fn create_scheduled_event(
    discord: &Arc<Client>,
    params: CreateScheduledEventParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    use twilight_model::guild::scheduled_event::PrivacyLevel;

    let guild_id = parse_id(&params.guild_id)?;

    let _privacy = match params.privacy_level {
        2 => PrivacyLevel::GuildOnly,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid privacy_level {other}. Valid: 2=guild_only"),
                None,
            ))
        }
    };

    let start_time = twilight_model::util::Timestamp::parse(&params.scheduled_start_time)
        .map_err(|_| rmcp::ErrorData::invalid_params("Invalid scheduled_start_time ISO8601 timestamp", None))?;

    let base = discord.create_guild_scheduled_event(guild_id, _privacy);

    match params.entity_type {
        1 => {
            // Stage instance — requires channel_id
            let channel_id = params.channel_id.as_deref()
                .ok_or_else(|| rmcp::ErrorData::invalid_params("channel_id required for stage_instance entity type", None))?;
            let channel_id = parse_id(channel_id)?;
            let mut req = base.stage_instance(channel_id, &params.name, &start_time);
            if let Some(ref desc) = params.description {
                req = req.description(desc);
            }
            let response = match req.await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            };
            match response.model().await {
                Ok(event) => json_result(&event),
                Err(e) => deserialize_error(e),
            }
        }
        2 => {
            // Voice — requires channel_id
            let channel_id = params.channel_id.as_deref()
                .ok_or_else(|| rmcp::ErrorData::invalid_params("channel_id required for voice entity type", None))?;
            let channel_id = parse_id(channel_id)?;
            let mut req = base.voice(channel_id, &params.name, &start_time);
            if let Some(ref desc) = params.description {
                req = req.description(desc);
            }
            let response = match req.await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            };
            match response.model().await {
                Ok(event) => json_result(&event),
                Err(e) => deserialize_error(e),
            }
        }
        3 => {
            // External — requires location and end time
            let location = params.location.as_deref()
                .ok_or_else(|| rmcp::ErrorData::invalid_params("location required for external entity type", None))?;
            let end_time_str = params.scheduled_end_time.as_deref()
                .ok_or_else(|| rmcp::ErrorData::invalid_params("scheduled_end_time required for external entity type", None))?;
            let end_time = twilight_model::util::Timestamp::parse(end_time_str)
                .map_err(|_| rmcp::ErrorData::invalid_params("Invalid scheduled_end_time ISO8601 timestamp", None))?;
            let mut req = base.external(&params.name, location, &start_time, &end_time);
            if let Some(ref desc) = params.description {
                req = req.description(desc);
            }
            let response = match req.await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            };
            match response.model().await {
                Ok(event) => json_result(&event),
                Err(e) => deserialize_error(e),
            }
        }
        other => Err(rmcp::ErrorData::invalid_params(
            format!("Invalid entity_type {other}. Valid: 1=stage_instance, 2=voice, 3=external"),
            None,
        )),
    }
}

// -- update_scheduled_event --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateScheduledEventParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The scheduled event ID
    pub event_id: String,
    /// New event name
    pub name: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New scheduled start time (ISO8601 timestamp)
    pub scheduled_start_time: Option<String>,
    /// New scheduled end time (ISO8601 timestamp)
    pub scheduled_end_time: Option<String>,
    /// New entity type: 1=stage_instance, 2=voice, 3=external
    pub entity_type: Option<u8>,
    /// New channel ID
    pub channel_id: Option<String>,
    /// New location (for external events)
    pub location: Option<String>,
    /// New status: 1=scheduled, 2=active, 3=completed, 4=canceled
    pub status: Option<u8>,
}

pub async fn update_scheduled_event(
    discord: &Arc<Client>,
    params: UpdateScheduledEventParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    use twilight_model::guild::scheduled_event::{EntityType, Status};

    let guild_id = parse_id(&params.guild_id)?;
    let event_id = parse_id(&params.event_id)?;

    let mut req = discord.update_guild_scheduled_event(guild_id, event_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(ref desc) = params.description {
        req = req.description(Some(desc.as_str()));
    }
    // Store parsed timestamps so references live long enough
    let start_ts;
    if let Some(ref start) = params.scheduled_start_time {
        start_ts = Some(twilight_model::util::Timestamp::parse(start)
            .map_err(|_| rmcp::ErrorData::invalid_params("Invalid scheduled_start_time", None))?);
        req = req.scheduled_start_time(start_ts.as_ref().unwrap());
    }
    let end_ts;
    if let Some(ref end) = params.scheduled_end_time {
        end_ts = Some(twilight_model::util::Timestamp::parse(end)
            .map_err(|_| rmcp::ErrorData::invalid_params("Invalid scheduled_end_time", None))?);
        req = req.scheduled_end_time(Some(end_ts.as_ref().unwrap()));
    }
    if let Some(et) = params.entity_type {
        let entity_type = match et {
            1 => EntityType::StageInstance,
            2 => EntityType::Voice,
            3 => EntityType::External,
            other => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Invalid entity_type {other}"),
                    None,
                ))
            }
        };
        req = req.entity_type(entity_type);
    }
    if let Some(ref cid) = params.channel_id {
        req = req.channel_id(parse_id(cid)?);
    }
    if let Some(ref loc) = params.location {
        req = req.location(Some(loc.as_str()));
    }
    if let Some(st) = params.status {
        let status = match st {
            1 => Status::Scheduled,
            2 => Status::Active,
            3 => Status::Completed,
            4 => Status::Cancelled,
            other => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Invalid status {other}. Valid: 1=scheduled, 2=active, 3=completed, 4=canceled"),
                    None,
                ))
            }
        };
        req = req.status(status);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(event) => json_result(&event),
        Err(e) => deserialize_error(e),
    }
}
