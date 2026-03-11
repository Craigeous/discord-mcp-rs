use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::channel::stage_instance::PrivacyLevel;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- create_stage_instance --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateStageInstanceParams {
    /// The stage channel ID
    pub channel_id: String,
    /// The topic of the stage instance (1-120 characters)
    pub topic: String,
    /// Privacy level: 2=guild_only (default)
    pub privacy_level: Option<u8>,
}

pub async fn create_stage_instance(
    discord: &Arc<Client>,
    params: CreateStageInstanceParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let privacy = match params.privacy_level.unwrap_or(2) {
        2 => PrivacyLevel::GuildOnly,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid privacy_level {other}. Valid: 2=guild_only"),
                None,
            ))
        }
    };

    let response = match discord
        .create_stage_instance(channel_id, &params.topic)
        .privacy_level(privacy)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(stage) => json_result(&stage),
        Err(e) => deserialize_error(e),
    }
}

// -- get_stage_instance --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetStageInstanceParams {
    /// The stage channel ID
    pub channel_id: String,
}

pub async fn get_stage_instance(
    discord: &Arc<Client>,
    params: GetStageInstanceParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.stage_instance(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(stage) => json_result(&stage),
        Err(e) => deserialize_error(e),
    }
}

// -- update_stage_instance --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateStageInstanceParams {
    /// The stage channel ID
    pub channel_id: String,
    /// New topic (1-120 characters)
    pub topic: Option<String>,
    /// Privacy level: 2=guild_only
    pub privacy_level: Option<u8>,
}

pub async fn update_stage_instance(
    discord: &Arc<Client>,
    params: UpdateStageInstanceParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let mut req = discord.update_stage_instance(channel_id);

    if let Some(ref topic) = params.topic {
        req = req.topic(topic);
    }
    if let Some(level) = params.privacy_level {
        let privacy = match level {
            2 => PrivacyLevel::GuildOnly,
            other => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Invalid privacy_level {other}. Valid: 2=guild_only"),
                    None,
                ))
            }
        };
        req = req.privacy_level(privacy);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(stage) => json_result(&stage),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_stage_instance --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteStageInstanceParams {
    /// The stage channel ID
    pub channel_id: String,
}

pub async fn delete_stage_instance(
    discord: &Arc<Client>,
    params: DeleteStageInstanceParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    match discord.delete_stage_instance(channel_id).await {
        Ok(_) => text_result("Stage instance deleted"),
        Err(e) => discord_api_error(e),
    }
}
