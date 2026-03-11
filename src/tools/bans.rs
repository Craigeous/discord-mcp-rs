use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_bans --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListBansParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_bans(
    discord: &Arc<Client>,
    params: ListBansParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.bans(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(bans) => json_result(&bans),
        Err(e) => deserialize_error(e),
    }
}

// -- get_ban --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetBanParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The banned user's ID
    pub user_id: String,
}

pub async fn get_ban(
    discord: &Arc<Client>,
    params: GetBanParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    let response = match discord.ban(guild_id, user_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(ban) => json_result(&ban),
        Err(e) => deserialize_error(e),
    }
}

// -- ban_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BanMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID to ban
    pub user_id: String,
    /// Number of seconds of messages to delete (0-604800, i.e. up to 7 days)
    pub delete_message_seconds: Option<u32>,
}

pub async fn ban_member(
    discord: &Arc<Client>,
    params: BanMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;

    let mut req = discord.create_ban(guild_id, user_id);

    if let Some(seconds) = params.delete_message_seconds {
        req = req.delete_message_seconds(seconds);
    }

    match req.await {
        Ok(_) => text_result("Member banned successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- unban_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UnbanMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID to unban
    pub user_id: String,
}

pub async fn unban_member(
    discord: &Arc<Client>,
    params: UnbanMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    match discord.delete_ban(guild_id, user_id).await {
        Ok(_) => text_result("Member unbanned successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- bulk_guild_ban (raw HTTP) --

use serde_json::Value;
use crate::server::DiscordMcpServer;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BulkGuildBanParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// List of user IDs to ban (max 200)
    pub user_ids: Vec<String>,
    /// Number of seconds of messages to delete (0-604800, i.e. up to 7 days)
    pub delete_message_seconds: Option<u32>,
}

pub async fn bulk_guild_ban(
    server: &DiscordMcpServer,
    params: BulkGuildBanParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut body = serde_json::json!({
        "user_ids": params.user_ids,
    });
    if let Some(seconds) = params.delete_message_seconds {
        body["delete_message_seconds"] = serde_json::json!(seconds);
    }

    let resp = server
        .raw_request(
            reqwest::Method::POST,
            &format!("/guilds/{}/bulk-ban", params.guild_id),
        )
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            crate::error::json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let msg = format!("Discord API error ({status}): {text}");
            tracing::warn!("{msg}");
            Ok(CallToolResult::error(vec![rmcp::model::Content::text(msg)]))
        }
        Err(e) => {
            let msg = format!("Request error: {e}");
            tracing::warn!("{msg}");
            Ok(CallToolResult::error(vec![rmcp::model::Content::text(msg)]))
        }
    }
}
