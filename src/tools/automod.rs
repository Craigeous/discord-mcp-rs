use std::sync::Arc;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use twilight_http::Client;

use crate::error::{discord_api_error, deserialize_error, json_result, text_result};
use crate::util::parse_id;

// -- list_automod_rules --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListAutomodRulesParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_automod_rules(
    discord: &Arc<Client>,
    params: ListAutomodRulesParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let response = match discord.auto_moderation_rules(guild_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.models().await {
        Ok(rules) => json_result(&rules),
        Err(e) => deserialize_error(e),
    }
}

// -- get_automod_rule --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAutomodRuleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The auto-moderation rule ID
    pub rule_id: String,
}

pub async fn get_automod_rule(
    discord: &Arc<Client>,
    params: GetAutomodRuleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let rule_id = parse_id(&params.rule_id)?;
    let response = match discord.auto_moderation_rule(guild_id, rule_id).await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(rule) => json_result(&rule),
        Err(e) => deserialize_error(e),
    }
}

// -- delete_automod_rule --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteAutomodRuleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The auto-moderation rule ID
    pub rule_id: String,
}

pub async fn delete_automod_rule(
    discord: &Arc<Client>,
    params: DeleteAutomodRuleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let guild_id = parse_id(&params.guild_id)?;
    let rule_id = parse_id(&params.rule_id)?;
    match discord.delete_auto_moderation_rule(guild_id, rule_id).await {
        Ok(_) => text_result("Auto-moderation rule deleted"),
        Err(e) => discord_api_error(e),
    }
}
