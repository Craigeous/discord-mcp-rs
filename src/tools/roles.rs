use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::guild::Permissions;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_roles --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListRolesParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_roles(
    discord: &Arc<Client>,
    params: ListRolesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.roles(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(roles) => json_result(&roles),
        Err(e) => deserialize_error(e),
    }
}

// -- create_role --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateRoleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Role name
    pub name: Option<String>,
    /// Role color as integer (decimal, e.g. 16711680 for red #FF0000)
    pub color: Option<u32>,
    /// Whether the role is displayed separately in the member list
    pub hoist: Option<bool>,
    /// Whether the role is mentionable
    pub mentionable: Option<bool>,
    /// Permission bit set as a string (decimal representation of the bitfield)
    pub permissions: Option<String>,
}

pub async fn create_role(
    discord: &Arc<Client>,
    params: CreateRoleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.create_role(guild_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(color) = params.color {
        req = req.color(color);
    }
    if let Some(hoist) = params.hoist {
        req = req.hoist(hoist);
    }
    if let Some(mentionable) = params.mentionable {
        req = req.mentionable(mentionable);
    }
    if let Some(ref perms) = params.permissions {
        let bits: u64 = perms.parse().map_err(|_| {
            rmcp::ErrorData::invalid_params("permissions must be a decimal number string", None)
        })?;
        req = req.permissions(Permissions::from_bits_truncate(bits));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(role) => json_result(&role),
        Err(e) => deserialize_error(e),
    }
}

// -- update_role --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateRoleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The role ID to update
    pub role_id: String,
    /// New role name
    pub name: Option<String>,
    /// New color as integer
    pub color: Option<u32>,
    /// Whether to display separately in member list
    pub hoist: Option<bool>,
    /// Whether the role is mentionable
    pub mentionable: Option<bool>,
    /// Permission bit set as a string
    pub permissions: Option<String>,
}

pub async fn update_role(
    discord: &Arc<Client>,
    params: UpdateRoleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let role_id = parse_id(&params.role_id)?;

    let mut req = discord.update_role(guild_id, role_id);

    if let Some(ref name) = params.name {
        req = req.name(Some(name.as_str()));
    }
    if let Some(color) = params.color {
        req = req.color(Some(color));
    }
    if let Some(hoist) = params.hoist {
        req = req.hoist(hoist);
    }
    if let Some(mentionable) = params.mentionable {
        req = req.mentionable(mentionable);
    }
    if let Some(ref perms) = params.permissions {
        let bits: u64 = perms.parse().map_err(|_| {
            rmcp::ErrorData::invalid_params("permissions must be a decimal number string", None)
        })?;
        req = req.permissions(Permissions::from_bits_truncate(bits));
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(role) => json_result(&role),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_role --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteRoleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The role ID to delete
    pub role_id: String,
}

pub async fn delete_role(
    discord: &Arc<Client>,
    params: DeleteRoleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let role_id = parse_id(&params.role_id)?;
    match discord.delete_role(guild_id, role_id).await {
        Ok(_) => text_result("Role deleted successfully"),
        Err(e) => discord_api_error(e),
    }
}

// -- add_role_to_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddRoleToMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID
    pub user_id: String,
    /// The role ID to add
    pub role_id: String,
}

pub async fn add_role_to_member(
    discord: &Arc<Client>,
    params: AddRoleToMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    let role_id = parse_id(&params.role_id)?;
    match discord.add_guild_member_role(guild_id, user_id, role_id).await {
        Ok(_) => text_result("Role added to member"),
        Err(e) => discord_api_error(e),
    }
}

// -- remove_role_from_member --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveRoleFromMemberParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The user ID
    pub user_id: String,
    /// The role ID to remove
    pub role_id: String,
}

pub async fn remove_role_from_member(
    discord: &Arc<Client>,
    params: RemoveRoleFromMemberParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let user_id = parse_id(&params.user_id)?;
    let role_id = parse_id(&params.role_id)?;
    match discord.remove_guild_member_role(guild_id, user_id, role_id).await {
        Ok(_) => text_result("Role removed from member"),
        Err(e) => discord_api_error(e),
    }
}
