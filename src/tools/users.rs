use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::{parse_id, read_file_as_data_uri};

// -- get_user --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetUserParams {
    /// The user ID
    pub user_id: String,
}

pub async fn get_user(
    discord: &Arc<Client>,
    params: GetUserParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let user_id = parse_id(&params.user_id)?;
    let response = match discord.user(user_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(user) => json_result(&user),
        Err(e) => deserialize_error(e),
    }
}

// -- create_dm --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateDmParams {
    /// The recipient user ID
    pub recipient_id: String,
}

pub async fn create_dm(
    discord: &Arc<Client>,
    params: CreateDmParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let recipient_id = parse_id(&params.recipient_id)?;
    let response = match discord.create_private_channel(recipient_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(channel) => json_result(&channel),
        Err(e) => deserialize_error(e),
    }
}

// -- leave_guild --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LeaveGuildParams {
    /// The guild (server) ID to leave
    pub guild_id: String,
}

pub async fn leave_guild(
    discord: &Arc<Client>,
    params: LeaveGuildParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    match discord.leave_guild(guild_id).await {
        Ok(_) => text_result("Left guild"),
        Err(e) => discord_api_error(e),
    }
}

// -- get_current_user_connections --

pub async fn get_current_user_connections(
    discord: &Arc<Client>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.current_user_connections().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(connections) => json_result(&connections),
        Err(e) => deserialize_error(e),
    }
}

// -- update_current_user --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateCurrentUserParams {
    /// New username
    pub username: Option<String>,
    /// Path to new avatar image file (png, jpg, gif, webp), or null to remove
    pub avatar_file_path: Option<String>,
}

pub async fn update_current_user(
    discord: &Arc<Client>,
    params: UpdateCurrentUserParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut req = discord.update_current_user();

    if let Some(ref username) = params.username {
        req = req.username(username);
    }

    let avatar_data;
    if let Some(ref path) = params.avatar_file_path {
        avatar_data = read_file_as_data_uri(path)?;
        req = req.avatar(Some(&avatar_data));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(user) => json_result(&user),
        Err(e) => deserialize_error(e),
    }
}
