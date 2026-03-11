use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- create_interaction_response --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInteractionResponseParams {
    /// The interaction ID
    pub interaction_id: String,
    /// The interaction token
    pub interaction_token: String,
    /// Response type: 1=pong, 4=channel_message, 5=deferred_channel_message, 6=deferred_update, 7=update_message
    pub response_type: u8,
    /// Message content (for types 4/7)
    pub content: Option<String>,
}

pub async fn create_interaction_response(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: CreateInteractionResponseParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let interaction_id = parse_id(&params.interaction_id)?;

    let response_type = match params.response_type {
        1 => twilight_model::http::interaction::InteractionResponseType::Pong,
        4 => twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
        5 => twilight_model::http::interaction::InteractionResponseType::DeferredChannelMessageWithSource,
        6 => twilight_model::http::interaction::InteractionResponseType::DeferredUpdateMessage,
        7 => twilight_model::http::interaction::InteractionResponseType::UpdateMessage,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid response_type {other}. Valid: 1=pong, 4=channel_message, 5=deferred_channel_message, 6=deferred_update, 7=update_message"),
                None,
            ))
        }
    };

    let data = if let Some(ref content) = params.content {
        Some(
            twilight_model::http::interaction::InteractionResponseData {
                content: Some(content.clone()),
                ..Default::default()
            },
        )
    } else {
        None
    };

    let response = twilight_model::http::interaction::InteractionResponse {
        kind: response_type,
        data,
    };

    match discord
        .interaction(application_id)
        .create_response(interaction_id, &params.interaction_token, &response)
        .await
    {
        Ok(_) => text_result("Interaction response sent"),
        Err(e) => discord_api_error(e),
    }
}

// -- get_original_response --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetOriginalResponseParams {
    /// The interaction token
    pub interaction_token: String,
}

pub async fn get_original_response(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: GetOriginalResponseParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord
        .interaction(application_id)
        .response(&params.interaction_token)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- edit_original_response --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditOriginalResponseParams {
    /// The interaction token
    pub interaction_token: String,
    /// New message content
    pub content: Option<String>,
}

pub async fn edit_original_response(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: EditOriginalResponseParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let interaction = discord.interaction(application_id);
    let mut req = interaction.update_response(&params.interaction_token);

    if let Some(ref content) = params.content {
        req = req.content(Some(content));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_original_response --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteOriginalResponseParams {
    /// The interaction token
    pub interaction_token: String,
}

pub async fn delete_original_response(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: DeleteOriginalResponseParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    match discord
        .interaction(application_id)
        .delete_response(&params.interaction_token)
        .await
    {
        Ok(_) => text_result("Original response deleted"),
        Err(e) => discord_api_error(e),
    }
}

// -- create_followup_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateFollowupMessageParams {
    /// The interaction token
    pub interaction_token: String,
    /// Message content
    pub content: String,
}

pub async fn create_followup_message(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: CreateFollowupMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord
        .interaction(application_id)
        .create_followup(&params.interaction_token)
        .content(&params.content)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- get_followup_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFollowupMessageParams {
    /// The interaction token
    pub interaction_token: String,
    /// The followup message ID
    pub message_id: String,
}

pub async fn get_followup_message(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: GetFollowupMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let message_id = parse_id(&params.message_id)?;
    let response = match discord
        .interaction(application_id)
        .followup(&params.interaction_token, message_id)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- edit_followup_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditFollowupMessageParams {
    /// The interaction token
    pub interaction_token: String,
    /// The followup message ID
    pub message_id: String,
    /// New message content
    pub content: Option<String>,
}

pub async fn edit_followup_message(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: EditFollowupMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let message_id = parse_id(&params.message_id)?;

    let interaction = discord.interaction(application_id);
    let mut req = interaction.update_followup(&params.interaction_token, message_id);

    if let Some(ref content) = params.content {
        req = req.content(Some(content));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_followup_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteFollowupMessageParams {
    /// The interaction token
    pub interaction_token: String,
    /// The followup message ID
    pub message_id: String,
}

pub async fn delete_followup_message(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: DeleteFollowupMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let message_id = parse_id(&params.message_id)?;
    match discord
        .interaction(application_id)
        .delete_followup(&params.interaction_token, message_id)
        .await
    {
        Ok(_) => text_result("Followup message deleted"),
        Err(e) => discord_api_error(e),
    }
}
