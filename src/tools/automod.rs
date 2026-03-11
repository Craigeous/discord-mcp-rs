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

// -- create_automod_rule --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateAutomodRuleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Rule name
    pub name: String,
    /// Event type: 1=message_send
    pub event_type: u8,
    /// Trigger type: 1=keyword, 3=spam, 4=keyword_preset, 5=mention_spam
    pub trigger_type: u8,
    /// List of keywords to match (for trigger_type 1)
    pub keyword_filter: Option<Vec<String>>,
    /// List of regex patterns to match (for trigger_type 1)
    pub regex_patterns: Option<Vec<String>>,
    /// Allow list of keywords exempt from triggering (for trigger_type 1 and 4)
    pub allow_list: Option<Vec<String>>,
    /// Keyword preset IDs (for trigger_type 4): 1=profanity, 2=sexual_content, 3=slurs
    pub presets: Option<Vec<u8>>,
    /// Total number of mentions allowed before triggering (for trigger_type 5)
    pub mention_total_limit: Option<u8>,
    /// Whether the rule is enabled
    pub enabled: Option<bool>,
}

pub async fn create_automod_rule(
    discord: &Arc<Client>,
    params: CreateAutomodRuleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    use twilight_model::guild::auto_moderation::{
        AutoModerationEventType, AutoModerationKeywordPresetType,
    };

    let guild_id = parse_id(&params.guild_id)?;

    let event_type = match params.event_type {
        1 => AutoModerationEventType::MessageSend,
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid event_type {other}. Valid: 1=message_send"),
                None,
            ))
        }
    };

    // Build up the base request (3 args: guild_id, name, event_type).
    // The trigger type is set via a terminator method (with_keyword, with_spam, etc.)
    // which also executes the request and returns a ResponseFuture.
    let mut req = discord.create_auto_moderation_rule(guild_id, &params.name, event_type);

    // Apply optional builder methods before calling the terminator
    req = req.action_block_message();
    if let Some(enabled) = params.enabled {
        req = req.enabled(enabled);
    }

    // Call the appropriate terminator based on trigger_type.
    // Each with_* method sets trigger_type, configures trigger_metadata, and executes.
    let response = match params.trigger_type {
        1 => {
            // Keyword trigger
            let keyword_filter_owned = params.keyword_filter.unwrap_or_default();
            let keyword_refs: Vec<&str> = keyword_filter_owned.iter().map(|s| s.as_str()).collect();
            let regex_owned = params.regex_patterns.unwrap_or_default();
            let regex_refs: Vec<&str> = regex_owned.iter().map(|s| s.as_str()).collect();
            let allow_owned = params.allow_list.unwrap_or_default();
            let allow_refs: Vec<&str> = allow_owned.iter().map(|s| s.as_str()).collect();
            match req.with_keyword(&keyword_refs, &regex_refs, &allow_refs).await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            }
        }
        3 => {
            // Spam trigger
            match req.with_spam().await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            }
        }
        4 => {
            // Keyword preset trigger
            let presets_raw = params.presets.unwrap_or_default();
            let preset_types: Vec<AutoModerationKeywordPresetType> = presets_raw
                .iter()
                .map(|p| match p {
                    1 => Ok(AutoModerationKeywordPresetType::Profanity),
                    2 => Ok(AutoModerationKeywordPresetType::SexualContent),
                    3 => Ok(AutoModerationKeywordPresetType::Slurs),
                    other => Err(rmcp::ErrorData::invalid_params(
                        format!("Invalid preset {other}. Valid: 1=profanity, 2=sexual_content, 3=slurs"),
                        None,
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let allow_owned = params.allow_list.unwrap_or_default();
            let allow_refs: Vec<&str> = allow_owned.iter().map(|s| s.as_str()).collect();
            match req.with_keyword_preset(&preset_types, &allow_refs).await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            }
        }
        5 => {
            // Mention spam trigger
            let limit = params.mention_total_limit.unwrap_or(5);
            match req.with_mention_spam(limit).await {
                Ok(r) => r,
                Err(e) => return discord_api_error(e),
            }
        }
        other => {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Invalid trigger_type {other}. Valid: 1=keyword, 3=spam, 4=keyword_preset, 5=mention_spam"),
                None,
            ))
        }
    };

    match response.model().await {
        Ok(rule) => json_result(&rule),
        Err(e) => deserialize_error(e),
    }
}

// -- update_automod_rule --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateAutomodRuleParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The auto-moderation rule ID
    pub rule_id: String,
    /// New rule name
    pub name: Option<String>,
    /// New event type: 1=message_send
    pub event_type: Option<u8>,
    /// New keyword filter list (for keyword trigger type)
    pub keyword_filter: Option<Vec<String>>,
    /// New regex patterns (for keyword trigger type)
    pub regex_patterns: Option<Vec<String>>,
    /// Allow list of keywords exempt from triggering
    pub allow_list: Option<Vec<String>>,
    /// Whether the rule is enabled
    pub enabled: Option<bool>,
}

pub async fn update_automod_rule(
    discord: &Arc<Client>,
    params: UpdateAutomodRuleParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    use twilight_model::guild::auto_moderation::{
        AutoModerationEventType, AutoModerationTriggerMetadata,
    };

    let guild_id = parse_id(&params.guild_id)?;
    let rule_id = parse_id(&params.rule_id)?;

    // Build trigger metadata if any trigger-related fields are provided
    let trigger_metadata = if params.keyword_filter.is_some()
        || params.regex_patterns.is_some()
        || params.allow_list.is_some()
    {
        Some(AutoModerationTriggerMetadata {
            allow_list: params.allow_list,
            keyword_filter: params.keyword_filter,
            presets: None,
            mention_raid_protection_enabled: None,
            mention_total_limit: None,
            regex_patterns: params.regex_patterns,
        })
    } else {
        None
    };

    let mut req = discord.update_auto_moderation_rule(guild_id, rule_id);

    if let Some(ref name) = params.name {
        req = req.name(name);
    }
    if let Some(et) = params.event_type {
        let event_type = match et {
            1 => AutoModerationEventType::MessageSend,
            other => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Invalid event_type {other}. Valid: 1=message_send"),
                    None,
                ))
            }
        };
        req = req.event_type(event_type);
    }
    if let Some(ref metadata) = trigger_metadata {
        req = req.trigger_metadata(metadata);
    }
    if let Some(enabled) = params.enabled {
        req = req.enabled(enabled);
    }

    let response = match req.await {
        Ok(r) => r,
        Err(e) => return discord_api_error(e),
    };
    match response.model().await {
        Ok(rule) => json_result(&rule),
        Err(e) => deserialize_error(e),
    }
}
