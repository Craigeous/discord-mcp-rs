use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_channel_webhooks --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListChannelWebhooksParams {
    /// The channel ID
    pub channel_id: String,
}

pub async fn list_channel_webhooks(
    discord: &Arc<Client>,
    params: ListChannelWebhooksParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.channel_webhooks(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(hooks) => json_result(&hooks),
        Err(e) => deserialize_error(e),
    }
}

// -- list_guild_webhooks --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildWebhooksParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_webhooks(
    discord: &Arc<Client>,
    params: ListGuildWebhooksParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.guild_webhooks(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(hooks) => json_result(&hooks),
        Err(e) => deserialize_error(e),
    }
}

// -- create_webhook --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateWebhookParams {
    /// The channel ID to create the webhook in
    pub channel_id: String,
    /// Webhook name (1-80 characters)
    pub name: String,
}

pub async fn create_webhook(
    discord: &Arc<Client>,
    params: CreateWebhookParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.create_webhook(channel_id, &params.name).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(hook) => json_result(&hook),
        Err(e) => deserialize_error(e),
    }
}

// -- get_webhook --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetWebhookParams {
    /// The webhook ID
    pub webhook_id: String,
}

pub async fn get_webhook(
    discord: &Arc<Client>,
    params: GetWebhookParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let webhook_id = parse_id(&params.webhook_id)?;
    let response = match discord.webhook(webhook_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(hook) => json_result(&hook),
        Err(e) => deserialize_error(e),
    }
}

// -- update_webhook --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateWebhookParams {
    /// The webhook ID
    pub webhook_id: String,
    /// New webhook name
    pub name: Option<String>,
    /// New channel ID to move the webhook to
    pub channel_id: Option<String>,
}

pub async fn update_webhook(
    discord: &Arc<Client>,
    params: UpdateWebhookParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let webhook_id = parse_id(&params.webhook_id)?;

    let mut req = discord.update_webhook(webhook_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(ref ch_id) = params.channel_id {
        req = req.channel_id(parse_id(ch_id)?);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(hook) => json_result(&hook),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_webhook --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteWebhookParams {
    /// The webhook ID
    pub webhook_id: String,
}

pub async fn delete_webhook(
    discord: &Arc<Client>,
    params: DeleteWebhookParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let webhook_id = parse_id(&params.webhook_id)?;
    match discord.delete_webhook(webhook_id).await {
        Ok(_) => text_result("Webhook deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- execute_webhook --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExecuteWebhookParams {
    /// The webhook ID
    pub webhook_id: String,
    /// The webhook token
    pub token: String,
    /// Message content
    pub content: Option<String>,
    /// Username override
    pub username: Option<String>,
    /// Avatar URL override
    pub avatar_url: Option<String>,
    /// Whether this is a TTS message
    pub tts: Option<bool>,
}

pub async fn execute_webhook(
    discord: &Arc<Client>,
    params: ExecuteWebhookParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let webhook_id = parse_id(&params.webhook_id)?;

    let mut req = discord.execute_webhook(webhook_id, &params.token);

    if let Some(ref content) = params.content {
        req = req.content(content);
    }
    if let Some(ref username) = params.username {
        req = req.username(username);
    }
    if let Some(ref avatar_url) = params.avatar_url {
        req = req.avatar_url(avatar_url);
    }
    if let Some(tts) = params.tts {
        req = req.tts(tts);
    }

    match req.await {
        Ok(_) => text_result("Webhook executed successfully"),
        Err(e) => discord_api_error(e),
    }
}
