use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::channel::ChannelType;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- list_guild_channels --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildChannelsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_channels(
    discord: &Arc<Client>,
    params: ListGuildChannelsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_channels(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(channels) => json_result(&channels),
        Err(e) => deserialize_error(e),
    }
}

// -- get_channel --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetChannelParams {
    /// The channel ID
    pub channel_id: String,
}

pub async fn get_channel(
    discord: &Arc<Client>,
    params: GetChannelParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.channel(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(channel) => json_result(&channel),
        Err(e) => deserialize_error(e),
    }
}

// -- create_channel --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateChannelParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Channel name (1-100 characters)
    pub name: String,
    /// Channel type: 0=text, 2=voice, 4=category, 5=announcement, 13=stage, 15=forum
    pub channel_type: Option<u8>,
    /// Channel topic (up to 1024 chars for text, 4096 for forum)
    pub topic: Option<String>,
    /// Whether the channel is NSFW
    pub nsfw: Option<bool>,
    /// Parent category channel ID
    pub parent_id: Option<String>,
    /// Sorting position of the channel
    pub position: Option<u64>,
}

fn parse_channel_type(kind: u8) -> Result<ChannelType, rmcp::ErrorData> {
    match kind {
        0 => Ok(ChannelType::GuildText),
        2 => Ok(ChannelType::GuildVoice),
        4 => Ok(ChannelType::GuildCategory),
        5 => Ok(ChannelType::GuildAnnouncement),
        13 => Ok(ChannelType::GuildStageVoice),
        15 => Ok(ChannelType::GuildForum),
        _ => Err(rmcp::ErrorData::invalid_params(
            format!("Invalid channel_type {kind}. Valid: 0=text, 2=voice, 4=category, 5=announcement, 13=stage, 15=forum"),
            None,
        )),
    }
}

pub async fn create_channel(
    discord: &Arc<Client>,
    params: CreateChannelParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.create_guild_channel(guild_id, &params.name);

    if let Some(kind) = params.channel_type {
        req = req.kind(parse_channel_type(kind)?);
    }
    if let Some(ref topic) = params.topic {
        req = req.topic(topic);
    }
    if let Some(nsfw) = params.nsfw {
        req = req.nsfw(nsfw);
    }
    if let Some(ref parent_id) = params.parent_id {
        req = req.parent_id(parse_id(parent_id)?);
    }
    if let Some(position) = params.position {
        req = req.position(position);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(channel) => json_result(&channel),
        Err(e) => deserialize_error(e),
    }
}

// -- update_channel --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateChannelParams {
    /// The channel ID to update
    pub channel_id: String,
    /// New channel name
    pub name: Option<String>,
    /// New channel topic
    pub topic: Option<String>,
    /// Whether the channel is NSFW
    pub nsfw: Option<bool>,
    /// New parent category channel ID (null to remove)
    pub parent_id: Option<String>,
    /// New sorting position
    pub position: Option<u64>,
}

pub async fn update_channel(
    discord: &Arc<Client>,
    params: UpdateChannelParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let mut req = discord.update_channel(channel_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(ref topic) = params.topic {
        req = req.topic(topic.as_str());
    }
    if let Some(nsfw) = params.nsfw {
        req = req.nsfw(nsfw);
    }
    if let Some(ref parent_id) = params.parent_id {
        req = req.parent_id(Some(parse_id(parent_id)?));
    }
    if let Some(position) = params.position {
        req = req.position(position);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(channel) => json_result(&channel),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_channel --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteChannelParams {
    /// The channel ID to delete
    pub channel_id: String,
}

pub async fn delete_channel(
    discord: &Arc<Client>,
    params: DeleteChannelParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    match discord.delete_channel(channel_id).await {
        Ok(_) => crate::error::text_result("Channel deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- update_channel_positions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChannelPosition {
    /// The channel ID
    pub id: String,
    /// The new position
    pub position: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateChannelPositionsParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Array of channel ID + position pairs
    pub positions: Vec<ChannelPosition>,
}

pub async fn update_channel_positions(
    discord: &Arc<Client>,
    params: UpdateChannelPositionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let twilight_positions: Vec<twilight_model::http::channel_position::Position> = params
        .positions
        .iter()
        .map(|p| {
            Ok(twilight_model::http::channel_position::Position {
                id: parse_id(&p.id)?,
                lock_permissions: None,
                parent_id: None,
                position: Some(Some(p.position)),
            })
        })
        .collect::<Result<Vec<_>, rmcp::ErrorData>>()?;

    match discord.update_guild_channel_positions(guild_id, &twilight_positions).await {
        Ok(_) => crate::error::text_result("Channel positions updated successfully"),
        Err(e) => discord_api_error(e),
    }
}
