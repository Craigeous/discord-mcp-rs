use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_channel_invites --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListChannelInvitesParams {
    /// The channel ID
    pub channel_id: String,
}

pub async fn list_channel_invites(
    discord: &Arc<Client>,
    params: ListChannelInvitesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.channel_invites(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(invites) => json_result(&invites),
        Err(e) => deserialize_error(e),
    }
}

// -- list_guild_invites --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildInvitesParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_invites(
    discord: &Arc<Client>,
    params: ListGuildInvitesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_invites(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(invites) => json_result(&invites),
        Err(e) => deserialize_error(e),
    }
}

// -- get_invite --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetInviteParams {
    /// The invite code (e.g. "abc123" from discord.gg/abc123)
    pub code: String,
    /// Whether to include approximate member counts
    pub with_counts: Option<bool>,
}

pub async fn get_invite(
    discord: &Arc<Client>,
    params: GetInviteParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut req = discord.invite(&params.code);

    if let Some(true) = params.with_counts {
        req = req.with_counts();
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(invite) => json_result(&invite),
        Err(e) => deserialize_error(e),
    }
}

// -- create_invite --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInviteParams {
    /// The channel ID to create the invite for
    pub channel_id: String,
    /// Max age in seconds (0 for never, default 86400 = 24h)
    pub max_age: Option<u32>,
    /// Max number of uses (0 for unlimited, default 0)
    pub max_uses: Option<u16>,
    /// Whether the invite is temporary (kicked when they disconnect if not assigned a role)
    pub temporary: Option<bool>,
    /// Whether to guarantee a unique invite code
    pub unique: Option<bool>,
}

pub async fn create_invite(
    discord: &Arc<Client>,
    params: CreateInviteParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let mut req = discord.create_invite(channel_id);

    if let Some(max_age) = params.max_age {
        req = req.max_age(max_age);
    }
    if let Some(max_uses) = params.max_uses {
        req = req.max_uses(max_uses);
    }
    if let Some(temporary) = params.temporary {
        req = req.temporary(temporary);
    }
    if let Some(unique) = params.unique {
        req = req.unique(unique);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(invite) => json_result(&invite),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_invite --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteInviteParams {
    /// The invite code to delete
    pub code: String,
}

pub async fn delete_invite(
    discord: &Arc<Client>,
    params: DeleteInviteParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    match discord.delete_invite(&params.code).await {
        Ok(_) => text_result("Invite deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}
