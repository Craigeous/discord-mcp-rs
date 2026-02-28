use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::channel::ChannelType;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- create_thread --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateThreadParams {
    /// The channel ID to create the thread in
    pub channel_id: String,
    /// Thread name
    pub name: String,
    /// Thread type: 11=public, 12=private
    pub thread_type: Option<u8>,
    /// Auto-archive duration in minutes (60, 1440, 4320, or 10080)
    pub auto_archive_duration: Option<u16>,
}

pub async fn create_thread(
    discord: &Arc<Client>,
    params: CreateThreadParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let kind = match params.thread_type.unwrap_or(11) {
        11 => ChannelType::PublicThread,
        12 => ChannelType::PrivateThread,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid thread_type {other}. Valid: 11=public, 12=private"),
                None,
            ))
        }
    };

    let mut req = discord.create_thread(channel_id, &params.name, kind);

    if let Some(duration) = params.auto_archive_duration {
        let d = twilight_model::channel::thread::AutoArchiveDuration::from(duration);
        req = req.auto_archive_duration(d);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(ch) => json_result(&ch),
        Err(e) => deserialize_error(e),
    }
}

// -- create_thread_from_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateThreadFromMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID to create a thread from
    pub message_id: String,
    /// Thread name
    pub name: String,
    /// Auto-archive duration in minutes (60, 1440, 4320, or 10080)
    pub auto_archive_duration: Option<u16>,
}

pub async fn create_thread_from_message(
    discord: &Arc<Client>,
    params: CreateThreadFromMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;

    let mut req = discord.create_thread_from_message(channel_id, message_id, &params.name);

    if let Some(duration) = params.auto_archive_duration {
        let d = twilight_model::channel::thread::AutoArchiveDuration::from(duration);
        req = req.auto_archive_duration(d);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(ch) => json_result(&ch),
        Err(e) => deserialize_error(e),
    }
}

// -- join_thread --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct JoinThreadParams {
    /// The thread channel ID
    pub channel_id: String,
}

pub async fn join_thread(
    discord: &Arc<Client>,
    params: JoinThreadParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    match discord.join_thread(channel_id).await {
        Ok(_) => text_result("Joined thread"),
        Err(e) => discord_api_error(e),
    }
}

// -- leave_thread --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LeaveThreadParams {
    /// The thread channel ID
    pub channel_id: String,
}

pub async fn leave_thread(
    discord: &Arc<Client>,
    params: LeaveThreadParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    match discord.leave_thread(channel_id).await {
        Ok(_) => text_result("Left thread"),
        Err(e) => discord_api_error(e),
    }
}

// -- add_thread_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddThreadMemberParams {
    /// The thread channel ID
    pub channel_id: String,
    /// The user ID to add
    pub user_id: String,
}

pub async fn add_thread_member(
    discord: &Arc<Client>,
    params: AddThreadMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let user_id = parse_id(&params.user_id)?;
    match discord.add_thread_member(channel_id, user_id).await {
        Ok(_) => text_result("Member added to thread"),
        Err(e) => discord_api_error(e),
    }
}

// -- remove_thread_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveThreadMemberParams {
    /// The thread channel ID
    pub channel_id: String,
    /// The user ID to remove
    pub user_id: String,
}

pub async fn remove_thread_member(
    discord: &Arc<Client>,
    params: RemoveThreadMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let user_id = parse_id(&params.user_id)?;
    match discord.remove_thread_member(channel_id, user_id).await {
        Ok(_) => text_result("Member removed from thread"),
        Err(e) => discord_api_error(e),
    }
}

// -- list_thread_members --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListThreadMembersParams {
    /// The thread channel ID
    pub channel_id: String,
}

pub async fn list_thread_members(
    discord: &Arc<Client>,
    params: ListThreadMembersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.thread_members(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(members) => json_result(&members),
        Err(e) => deserialize_error(e),
    }
}

// -- list_active_threads --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListActiveThreadsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_active_threads(
    discord: &Arc<Client>,
    params: ListActiveThreadsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.active_threads(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(threads) => json_result(&threads),
        Err(e) => deserialize_error(e),
    }
}

// -- list_public_archived_threads --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPublicArchivedThreadsParams {
    /// The parent channel ID
    pub channel_id: String,
}

pub async fn list_public_archived_threads(
    discord: &Arc<Client>,
    params: ListPublicArchivedThreadsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.public_archived_threads(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(threads) => json_result(&threads),
        Err(e) => deserialize_error(e),
    }
}
