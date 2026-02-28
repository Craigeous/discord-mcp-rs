use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result};
use crate::util::parse_id;

// -- get_audit_log --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAuditLogParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Filter by user who performed the action
    pub user_id: Option<String>,
    /// Filter by action type (integer, see Discord docs for values)
    pub action_type: Option<u16>,
    /// Get entries before this audit log entry ID
    pub before: Option<String>,
    /// Max number of entries to return (1-100, default 50)
    pub limit: Option<u16>,
}

pub async fn get_audit_log(
    discord: &Arc<Client>,
    params: GetAuditLogParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.audit_log(guild_id);

    if let Some(ref user_id) = params.user_id {
        req = req.user_id(parse_id(user_id)?);
    }
    if let Some(action_type) = params.action_type {
        // twilight expects AuditLogEventType, but we'll use the raw u16
        // by converting through the numeric representation
        let event_type = twilight_model::guild::audit_log::AuditLogEventType::from(action_type);
        req = req.action_type(event_type);
    }
    if let Some(ref before) = params.before {
        let id: u64 = before.trim().parse().map_err(|_| {
            rmcp::ErrorData::invalid_params(format!("Invalid ID '{before}'"), None)
        })?;
        req = req.before(id);
    }
    if let Some(limit) = params.limit {
        req = req.limit(limit);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(log) => json_result(&log),
        Err(e) => deserialize_error(e),
    }
}
