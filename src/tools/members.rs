use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_http::request::AuditLogReason;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_guild_members --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildMembersParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Max number of members to return (1-1000, default 1000)
    pub limit: Option<u16>,
    /// Get members after this user ID (for pagination)
    pub after: Option<String>,
}

pub async fn list_guild_members(
    discord: &Arc<Client>,
    params: ListGuildMembersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.guild_members(guild_id);

    if let Some(limit) = params.limit {
        req = req.limit(limit);
    }
    if let Some(ref after) = params.after {
        req = req.after(parse_id(after)?);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(members) => json_result(&members),
        Err(e) => deserialize_error(e),
    }
}

// -- get_guild_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID
    pub user_id: String,
}

pub async fn get_guild_member(
    discord: &Arc<Client>,
    params: GetGuildMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    let response = match discord.guild_member(guild_id, user_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(member) => json_result(&member),
        Err(e) => deserialize_error(e),
    }
}

// -- search_guild_members --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchGuildMembersParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Query string to match against usernames and nicknames
    pub query: String,
    /// Max number of members to return (1-1000, default 1)
    pub limit: Option<u16>,
}

pub async fn search_guild_members(
    discord: &Arc<Client>,
    params: SearchGuildMembersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.search_guild_members(guild_id, &params.query);

    if let Some(limit) = params.limit {
        req = req.limit(limit);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(members) => json_result(&members),
        Err(e) => deserialize_error(e),
    }
}

// -- update_guild_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGuildMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID to update
    pub user_id: String,
    /// New nickname (empty string to remove)
    pub nick: Option<String>,
    /// Array of role IDs to set on the member
    pub roles: Option<Vec<String>>,
    /// Whether the user is muted in voice channels
    pub mute: Option<bool>,
    /// Whether the user is deafened in voice channels
    pub deaf: Option<bool>,
    /// Voice channel ID to move the user to (null to disconnect)
    pub channel_id: Option<String>,
}

pub async fn update_guild_member(
    discord: &Arc<Client>,
    params: UpdateGuildMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;

    let mut req = discord.update_guild_member(guild_id, user_id);

    if let Some(ref nick) = params.nick {
        req = req.nick(Some(nick));
    }
    let role_ids: Option<Vec<_>> = match params.roles {
        Some(ref roles) => Some(
            roles
                .iter()
                .map(|id| parse_id(id))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };
    if let Some(ref ids) = role_ids {
        req = req.roles(ids);
    }
    if let Some(mute) = params.mute {
        req = req.mute(mute);
    }
    if let Some(deaf) = params.deaf {
        req = req.deaf(deaf);
    }
    if let Some(ref channel_id) = params.channel_id {
        req = req.channel_id(Some(parse_id(channel_id)?));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(member) => json_result(&member),
        Err(e) => deserialize_error(e),
    }
}

// -- kick_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KickMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID to kick
    pub user_id: String,
    /// Reason for the kick (shows in audit log)
    pub reason: Option<String>,
}

pub async fn kick_member(
    discord: &Arc<Client>,
    params: KickMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;

    let mut req = discord.remove_guild_member(guild_id, user_id);
    if let Some(ref reason) = params.reason {
        req = req.reason(reason);
    }

    match req.await {
        Ok(_) => text_result("Member kicked successfully"),
        Err(e) => discord_api_error(e),
    }
}
