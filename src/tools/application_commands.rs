use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_global_commands --

pub async fn list_global_commands(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.interaction(application_id).global_commands().await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(cmds) => json_result(&cmds),
        Err(e) => deserialize_error(e),
    }
}

// -- create_global_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGlobalCommandParams {
    /// Command name (1-32 characters, lowercase)
    pub name: String,
    /// Command description (1-100 characters)
    pub description: String,
}

pub async fn create_global_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: CreateGlobalCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord
        .interaction(application_id)
        .create_global_command()
        .chat_input(&params.name, &params.description)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(cmd) => json_result(&cmd),
        Err(e) => deserialize_error(e),
    }
}

// -- update_global_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGlobalCommandParams {
    /// The command ID to update
    pub command_id: String,
    /// New command name
    pub name: Option<String>,
    /// New command description
    pub description: Option<String>,
}

pub async fn update_global_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: UpdateGlobalCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let command_id = parse_id(&params.command_id)?;

    let interaction = discord.interaction(application_id);
    let mut req = interaction.update_global_command(command_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(ref desc) = params.description {
        req = req.description(desc);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(cmd) => json_result(&cmd),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_global_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteGlobalCommandParams {
    /// The command ID to delete
    pub command_id: String,
}

pub async fn delete_global_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: DeleteGlobalCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let command_id = parse_id(&params.command_id)?;
    match discord
        .interaction(application_id)
        .delete_global_command(command_id)
        .await
    {
        Ok(_) => text_result("Global command deleted"),
        Err(e) => discord_api_error(e),
    }
}

// -- list_guild_commands --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildCommandsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_commands(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: ListGuildCommandsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord
        .interaction(application_id)
        .guild_commands(guild_id)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(cmds) => json_result(&cmds),
        Err(e) => deserialize_error(e),
    }
}

// -- create_guild_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGuildCommandParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Command name (1-32 characters, lowercase)
    pub name: String,
    /// Command description (1-100 characters)
    pub description: String,
}

pub async fn create_guild_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: CreateGuildCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord
        .interaction(application_id)
        .create_guild_command(guild_id)
        .chat_input(&params.name, &params.description)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(cmd) => json_result(&cmd),
        Err(e) => deserialize_error(e),
    }
}

// -- update_guild_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGuildCommandParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The command ID to update
    pub command_id: String,
    /// New command name
    pub name: Option<String>,
    /// New command description
    pub description: Option<String>,
}

pub async fn update_guild_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: UpdateGuildCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let command_id = parse_id(&params.command_id)?;

    let interaction = discord.interaction(application_id);
    let mut req = interaction.update_guild_command(guild_id, command_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(ref desc) = params.description {
        req = req.description(desc);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(cmd) => json_result(&cmd),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_guild_command --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteGuildCommandParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The command ID to delete
    pub command_id: String,
}

pub async fn delete_guild_command(
    discord: &Arc<Client>,
    application_id: Id<ApplicationMarker>,
    params: DeleteGuildCommandParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let command_id = parse_id(&params.command_id)?;
    match discord
        .interaction(application_id)
        .delete_guild_command(guild_id, command_id)
        .await
    {
        Ok(_) => text_result("Guild command deleted"),
        Err(e) => discord_api_error(e),
    }
}
