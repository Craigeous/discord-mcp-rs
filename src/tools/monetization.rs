use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::Deserialize;
use serde_json::Value;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::error::{json_result, text_result};
use crate::server::DiscordMcpServer;

fn raw_error(msg: impl Into<String>) -> Result<CallToolResult, rmcp::ErrorData> {
    let msg = msg.into();
    tracing::warn!("{msg}");
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(msg)]))
}

// -- list_skus --

pub async fn list_skus(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::GET,
            &format!("/applications/{}/skus", application_id),
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

// -- list_entitlements --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListEntitlementsParams {
    /// User ID to filter by
    pub user_id: Option<String>,
    /// SKU IDs to filter by (comma-separated)
    pub sku_ids: Option<String>,
    /// Guild ID to filter by
    pub guild_id: Option<String>,
    /// Max number of entitlements to return (1-100, default 100)
    pub limit: Option<u8>,
    /// Whether to exclude ended entitlements
    pub exclude_ended: Option<bool>,
}

pub async fn list_entitlements(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
    params: ListEntitlementsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut req = server.raw_request(
        reqwest::Method::GET,
        &format!("/applications/{}/entitlements", application_id),
    );

    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(ref uid) = params.user_id {
        query.push(("user_id".into(), uid.clone()));
    }
    if let Some(ref sids) = params.sku_ids {
        query.push(("sku_ids".into(), sids.clone()));
    }
    if let Some(ref gid) = params.guild_id {
        query.push(("guild_id".into(), gid.clone()));
    }
    if let Some(limit) = params.limit {
        query.push(("limit".into(), limit.to_string()));
    }
    if let Some(exclude) = params.exclude_ended {
        query.push(("exclude_ended".into(), exclude.to_string()));
    }
    if !query.is_empty() {
        req = req.query(&query);
    }

    let resp = req.send().await;

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

// -- get_entitlement --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetEntitlementParams {
    /// The entitlement ID
    pub entitlement_id: String,
}

pub async fn get_entitlement(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
    params: GetEntitlementParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::GET,
            &format!(
                "/applications/{}/entitlements/{}",
                application_id, params.entitlement_id
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

// -- create_test_entitlement --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTestEntitlementParams {
    /// The SKU ID
    pub sku_id: String,
    /// The owner ID (user or guild)
    pub owner_id: String,
    /// Owner type: 1=guild, 2=user
    pub owner_type: u8,
}

pub async fn create_test_entitlement(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
    params: CreateTestEntitlementParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let body = serde_json::json!({
        "sku_id": params.sku_id,
        "owner_id": params.owner_id,
        "owner_type": params.owner_type,
    });

    let resp = server
        .raw_request(
            reqwest::Method::POST,
            &format!("/applications/{}/entitlements", application_id),
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

// -- delete_test_entitlement --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteTestEntitlementParams {
    /// The entitlement ID to delete
    pub entitlement_id: String,
}

pub async fn delete_test_entitlement(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
    params: DeleteTestEntitlementParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::DELETE,
            &format!(
                "/applications/{}/entitlements/{}",
                application_id, params.entitlement_id
            ),
        )
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => text_result("Test entitlement deleted"),
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- consume_entitlement --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConsumeEntitlementParams {
    /// The entitlement ID to consume
    pub entitlement_id: String,
}

pub async fn consume_entitlement(
    server: &DiscordMcpServer,
    application_id: Id<ApplicationMarker>,
    params: ConsumeEntitlementParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let resp = server
        .raw_request(
            reqwest::Method::POST,
            &format!(
                "/applications/{}/entitlements/{}/consume",
                application_id, params.entitlement_id
            ),
        )
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => text_result("Entitlement consumed"),
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            raw_error(format!("Discord API error ({status}): {text}"))
        }
        Err(e) => raw_error(format!("Request error: {e}")),
    }
}

// -- list_sku_subscriptions --

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSkuSubscriptionsParams {
    /// The SKU ID
    pub sku_id: String,
    /// Max results (1-100, default 100)
    pub limit: Option<u8>,
    /// Get subscriptions after this ID (for pagination)
    pub after: Option<String>,
    /// Get subscriptions before this ID (for pagination)
    pub before: Option<String>,
    /// User ID to filter by
    pub user_id: Option<String>,
}

pub async fn list_sku_subscriptions(
    server: &DiscordMcpServer,
    params: ListSkuSubscriptionsParams,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut req = server.raw_request(
        reqwest::Method::GET,
        &format!("/skus/{}/subscriptions", params.sku_id),
    );

    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(limit) = params.limit {
        query.push(("limit".into(), limit.to_string()));
    }
    if let Some(ref after) = params.after {
        query.push(("after".into(), after.clone()));
    }
    if let Some(ref before) = params.before {
        query.push(("before".into(), before.clone()));
    }
    if let Some(ref uid) = params.user_id {
        query.push(("user_id".into(), uid.clone()));
    }
    if !query.is_empty() {
        req = req.query(&query);
    }

    let resp = req.send().await;

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
