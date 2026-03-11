use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- get_template --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTemplateParams {
    /// The template code
    pub template_code: String,
}

pub async fn get_template(
    discord: &Arc<Client>,
    params: GetTemplateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let response = match discord.get_template(&params.template_code).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(template) => json_result(&template),
        Err(e) => deserialize_error(e),
    }
}

// -- list_guild_templates --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildTemplatesParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_templates(
    discord: &Arc<Client>,
    params: ListGuildTemplatesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.get_templates(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(templates) => json_result(&templates),
        Err(e) => deserialize_error(e),
    }
}

// -- create_guild_template --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGuildTemplateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Template name (1-100 characters)
    pub name: String,
    /// Template description (up to 120 characters)
    pub description: Option<String>,
}

pub async fn create_guild_template(
    discord: &Arc<Client>,
    params: CreateGuildTemplateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.create_template(guild_id, &params.name);

    if let Some(ref desc) = params.description {
        req = req.description(desc);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(template) => json_result(&template),
        Err(e) => deserialize_error(e),
    }
}

// -- sync_guild_template --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SyncGuildTemplateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The template code
    pub template_code: String,
}

pub async fn sync_guild_template(
    discord: &Arc<Client>,
    params: SyncGuildTemplateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord
        .sync_template(guild_id, &params.template_code)
        .await
    {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(template) => json_result(&template),
        Err(e) => deserialize_error(e),
    }
}

// -- update_guild_template --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGuildTemplateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The template code
    pub template_code: String,
    /// New template name
    pub name: Option<String>,
    /// New template description
    pub description: Option<String>,
}

pub async fn update_guild_template(
    discord: &Arc<Client>,
    params: UpdateGuildTemplateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;

    let mut req = discord.update_template(guild_id, &params.template_code);

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
        Ok(template) => json_result(&template),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_guild_template --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteGuildTemplateParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The template code
    pub template_code: String,
}

pub async fn delete_guild_template(
    discord: &Arc<Client>,
    params: DeleteGuildTemplateParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    match discord
        .delete_template(guild_id, &params.template_code)
        .await
    {
        Ok(_) => text_result("Template deleted"),
        Err(e) => discord_api_error(e),
    }
}
