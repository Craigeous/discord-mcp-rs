use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- get_poll_answer_voters --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPollAnswerVotersParams {
    /// The channel ID containing the poll message
    pub channel_id: String,
    /// The message ID of the poll
    pub message_id: String,
    /// The answer ID to get voters for
    pub answer_id: u8,
    /// Max number of users to return (1-100, default 25)
    pub limit: Option<u8>,
    /// Get users after this user ID (for pagination)
    pub after: Option<String>,
}

pub async fn get_poll_answer_voters(
    discord: &Arc<Client>,
    params: GetPollAnswerVotersParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;

    let mut req = discord.get_answer_voters(channel_id, message_id, params.answer_id);

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
    match response.model().await {
        Ok(voters) => json_result(&voters),
        Err(e) => deserialize_error(e),
    }
}

// -- end_poll --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EndPollParams {
    /// The channel ID containing the poll message
    pub channel_id: String,
    /// The message ID of the poll to end
    pub message_id: String,
}

pub async fn end_poll(
    discord: &Arc<Client>,
    params: EndPollParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let message_id = parse_id(&params.message_id)?;
    let response = match discord.end_poll(channel_id, message_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(msg) => json_result(&msg),
        Err(e) => deserialize_error(e),
    }
}
