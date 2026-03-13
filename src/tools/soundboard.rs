use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{json_result, text_result};
use crate::server::DiscordMcpServer;
use crate::util::read_file_as_data_uri;

fn raw_error(msg: impl Into<String>) -> Result<CallToolResult, rmcp::ErrorData> {
    let msg = msg.into();
    tracing::warn!("{msg}");
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(msg)]))
}

// -- send_soundboard_sound --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendSoundboardSoundParams {
    /// The voice channel ID to play the sound in
    pub channel_id: String,
    /// The soundboard sound ID
    pub sound_id: String,
    /// The source guild ID of the sound (for default sounds, omit)
    pub source_guild_id: Option<String>,
}

pub async fn send_soundboard_sound(
    server: &DiscordMcpServer,
    params: SendSoundboardSoundParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut body = serde_json::json!({
        "sound_id": params.sound_id,
    });
    if let Some(ref gid) = params.source_guild_id {
        body["source_guild_id"] = Value::String(gid.clone());
    }

    let resp = server
        .raw_request(
            reqwest::Method::POST,
            &format!("/channels/{}/send-soundboard-sound", params.channel_id),
        )
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => text_result("Soundboard sound sent"),
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- list_default_soundboard_sounds --

pub async fn list_default_soundboard_sounds(
    server: &DiscordMcpServer,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(reqwest::Method::GET, "/soundboard-default-sounds")
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- list_guild_soundboard_sounds --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListGuildSoundboardSoundsParams {
    /// The guild (server) ID
    pub guild_id: String,
}

pub async fn list_guild_soundboard_sounds(
    server: &DiscordMcpServer,
    params: ListGuildSoundboardSoundsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::GET,
            &format!("/guilds/{}/soundboard-sounds", params.guild_id),
        )
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- get_guild_soundboard_sound --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetGuildSoundboardSoundParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The soundboard sound ID
    pub sound_id: String,
}

pub async fn get_guild_soundboard_sound(
    server: &DiscordMcpServer,
    params: GetGuildSoundboardSoundParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::GET,
            &format!(
                "/guilds/{}/soundboard-sounds/{}",
                params.guild_id, params.sound_id
            ),
        )
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- create_guild_soundboard_sound --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateGuildSoundboardSoundParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// Sound name (2-32 characters)
    pub name: String,
    /// Path to the audio file (mp3 or ogg, max 512KB)
    pub file_path: String,
    /// Volume of the sound (0.0 to 1.0, default 1.0)
    pub volume: Option<f64>,
    /// Unicode emoji for the sound
    pub emoji_name: Option<String>,
    /// Custom emoji ID for the sound
    pub emoji_id: Option<String>,
}

pub async fn create_guild_soundboard_sound(
    server: &DiscordMcpServer,
    params: CreateGuildSoundboardSoundParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let sound = read_file_as_data_uri(&params.file_path)?;

    let mut body = serde_json::json!({
        "name": params.name,
        "sound": sound,
    });
    if let Some(vol) = params.volume {
        body["volume"] = serde_json::json!(vol);
    }
    if let Some(ref name) = params.emoji_name {
        body["emoji_name"] = Value::String(name.clone());
    }
    if let Some(ref id) = params.emoji_id {
        body["emoji_id"] = Value::String(id.clone());
    }

    let resp = server
        .raw_request(
            reqwest::Method::POST,
            &format!("/guilds/{}/soundboard-sounds", params.guild_id),
        )
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- update_guild_soundboard_sound --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateGuildSoundboardSoundParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The soundboard sound ID
    pub sound_id: String,
    /// New sound name (2-32 characters)
    pub name: Option<String>,
    /// New volume (0.0 to 1.0)
    pub volume: Option<f64>,
    /// Unicode emoji for the sound
    pub emoji_name: Option<String>,
    /// Custom emoji ID for the sound
    pub emoji_id: Option<String>,
}

pub async fn update_guild_soundboard_sound(
    server: &DiscordMcpServer,
    params: UpdateGuildSoundboardSoundParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut body = serde_json::json!({});
    if let Some(ref name) = params.name {
        body["name"] = Value::String(name.clone());
    }
    if let Some(vol) = params.volume {
        body["volume"] = serde_json::json!(vol);
    }
    if let Some(ref name) = params.emoji_name {
        body["emoji_name"] = Value::String(name.clone());
    }
    if let Some(ref id) = params.emoji_id {
        body["emoji_id"] = Value::String(id.clone());
    }

    let resp = server
        .raw_request(
            reqwest::Method::PATCH,
            &format!(
                "/guilds/{}/soundboard-sounds/{}",
                params.guild_id, params.sound_id
            ),
        )
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.map_err(|e| {
                rmcp::ErrorData::internal_error(format!("JSON parse error: {e}"), None)
            })?;
            json_result(&data)
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- delete_guild_soundboard_sound --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteGuildSoundboardSoundParams {
    /// The guild (server) ID
    pub guild_id: String,
    /// The soundboard sound ID
    pub sound_id: String,
}

pub async fn delete_guild_soundboard_sound(
    server: &DiscordMcpServer,
    params: DeleteGuildSoundboardSoundParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::DELETE,
            &format!(
                "/guilds/{}/soundboard-sounds/{}",
                params.guild_id, params.sound_id
            ),
        )
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => text_result("Soundboard sound deleted"),
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}
