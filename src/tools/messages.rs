use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::MessageMarker;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_messages --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListMessagesParams {
    /// The channel ID to get messages from
    pub channel_id: String,
    /// Max number of messages to return (1-100, default 50)
    pub limit: Option<u16>,
    /// Get messages before this message ID
    pub before: Option<String>,
    /// Get messages after this message ID
    pub after: Option<String>,
    /// Get messages around this message ID
    pub around: Option<String>,
}

pub async fn list_messages(
    discord: &Arc<Client>,
    params: ListMessagesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let base = discord.channel_messages(channel_id);

    // .before/.after/.around change the return type, so we must branch
    let result = if let Some(ref before) = params.before {
        let mut req = base.before(parse_id::<MessageMarker>(before)?);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }
        req.await
    } else if let Some(ref after) = params.after {
        let mut req = base.after(parse_id::<MessageMarker>(after)?);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }
        req.await
    } else if let Some(ref around) = params.around {
        let mut req = base.around(parse_id::<MessageMarker>(around)?);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }
        req.await
    } else {
        let mut req = base;
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }
        req.await
    };

    let response = match result {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(messages) => json_result(&messages),
        Err(e) => deserialize_error(e),
    }
}

// -- get_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID
    pub message_id: String,
}

pub async fn get_message(
    discord: &Arc<Client>,
    params: GetMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let response = match discord.message(channel_id, message_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}

// -- send_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendMessageParams {
    /// The channel ID to send the message to
    pub channel_id: String,
    /// Text content (up to 2000 characters)
    pub content: Option<String>,
    /// Whether this is a TTS message
    pub tts: Option<bool>,
    /// Message ID to reply to
    pub reply_to: Option<String>,
}

pub async fn send_message(
    discord: &Arc<Client>,
    params: SendMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;

    let mut req = discord.create_message(channel_id);

    if let Some(ref content) = params.content {
        req = req.content(content);
    }
    if let Some(tts) = params.tts {
        req = req.tts(tts);
    }
    if let Some(ref reply_to) = params.reply_to {
        req = req.reply(parse_id(reply_to)?);
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

// -- edit_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID to edit
    pub message_id: String,
    /// New text content
    pub content: Option<String>,
}

pub async fn edit_message(
    discord: &Arc<Client>,
    params: EditMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;

    let mut req = discord.update_message(channel_id, message_id);

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

// -- delete_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID to delete
    pub message_id: String,
}

pub async fn delete_message(
    discord: &Arc<Client>,
    params: DeleteMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    match discord.delete_message(channel_id, message_id).await {
        Ok(_) => text_result("Message deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- bulk_delete_messages --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BulkDeleteMessagesParams {
    /// The channel ID
    pub channel_id: String,
    /// Array of message IDs to delete (2-100 messages, must be < 14 days old)
    pub message_ids: Vec<String>,
}

pub async fn bulk_delete_messages(
    discord: &Arc<Client>,
    params: BulkDeleteMessagesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let ids: Vec<Id<MessageMarker>> = params
        .message_ids
        .iter()
        .map(|id| parse_id(id))
        .collect::<Result<Vec<_>, _>>()?;

    match discord.delete_messages(channel_id, &ids).await {
        Ok(_) => text_result(format!("{} messages deleted successfully", ids.len())),
        Err(e) => discord_api_error(e),
    }
}

// -- pin_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PinMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID to pin
    pub message_id: String,
}

pub async fn pin_message(
    discord: &Arc<Client>,
    params: PinMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    match discord.create_pin(channel_id, message_id).await {
        Ok(_) => text_result("Message pinned successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- unpin_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UnpinMessageParams {
    /// The channel ID
    pub channel_id: String,
    /// The message ID to unpin
    pub message_id: String,
}

pub async fn unpin_message(
    discord: &Arc<Client>,
    params: UnpinMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    match discord.delete_pin(channel_id, message_id).await {
        Ok(_) => text_result("Message unpinned successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- get_pinned_messages --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPinnedMessagesParams {
    /// The channel ID
    pub channel_id: String,
}

pub async fn get_pinned_messages(
    discord: &Arc<Client>,
    params: GetPinnedMessagesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let response = match discord.pins(channel_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msgs) => json_result(&msgs),
        Err(e) => deserialize_error(e),
    }
}

// -- crosspost_message --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CrosspostMessageParams {
    /// The announcement channel ID
    pub channel_id: String,
    /// The message ID to crosspost
    pub message_id: String,
}

pub async fn crosspost_message(
    discord: &Arc<Client>,
    params: CrosspostMessageParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let response = match discord.crosspost_message(channel_id, message_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}
