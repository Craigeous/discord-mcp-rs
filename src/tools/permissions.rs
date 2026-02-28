use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::guild::Permissions;
use twilight_model::http::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};

use crate::error::{discord_api_error, text_result};
use crate::util::parse_id;

// -- update_channel_permission --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateChannelPermissionParams {
    /// The channel ID
    pub channel_id: String,
    /// The target ID (role or user ID)
    pub target_id: String,
    /// Target type: "role" or "member"
    pub target_type: String,
    /// Allowed permissions bit set as string (decimal)
    pub allow: Option<String>,
    /// Denied permissions bit set as string (decimal)
    pub deny: Option<String>,
}

fn parse_permissions(s: &str) -> Result<Permissions, rmcp::ErrorData> {
    let bits: u64 = s.parse().map_err(|_| {
        rmcp::ErrorData::invalid_params("Permissions must be a decimal number string", None)
    })?;
    Ok(Permissions::from_bits_truncate(bits))
}

pub async fn update_channel_permission(
    discord: &Arc<Client>,
    params: UpdateChannelPermissionParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let target_id = parse_id(&params.target_id)?;

    let kind = match params.target_type.as_str() {
        "role" => PermissionOverwriteType::Role,
        "member" => PermissionOverwriteType::Member,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid target_type '{other}'. Must be 'role' or 'member'"),
                None,
            ))
        }
    };

    let allow = match params.allow {
        Some(ref s) => parse_permissions(s)?,
        None => Permissions::empty(),
    };
    let deny = match params.deny {
        Some(ref s) => parse_permissions(s)?,
        None => Permissions::empty(),
    };

    let overwrite = PermissionOverwrite {
        id: target_id,
        kind,
        allow: Some(allow),
        deny: Some(deny),
    };

    match discord
        .update_channel_permission(channel_id, &overwrite)
        .await
    {
        Ok(_) => text_result("Channel permission updated"),
        Err(e) => discord_api_error(e),
    }
}

// -- delete_channel_permission --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteChannelPermissionParams {
    /// The channel ID
    pub channel_id: String,
    /// The target ID (role or user ID) whose permission overwrite to delete
    pub target_id: String,
    /// Target type: "role" or "member"
    pub target_type: String,
}

pub async fn delete_channel_permission(
    discord: &Arc<Client>,
    params: DeleteChannelPermissionParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let channel_id = parse_id(&params.channel_id)?;
    let base = discord.delete_channel_permission(channel_id);

    let result = match params.target_type.as_str() {
        "role" => base.role(parse_id(&params.target_id)?).await,
        "member" => base.member(parse_id(&params.target_id)?).await,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid target_type '{other}'. Must be 'role' or 'member'"),
                None,
            ))
        }
    };

    match result {
        Ok(_) => text_result("Channel permission overwrite deleted"),
        Err(e) => discord_api_error(e),
    }
}
