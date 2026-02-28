use rmcp::model::{CallToolResult, Content};

/// Convert a twilight-http error into an MCP tool result with error text.
/// Returns as a successful tool result so the LLM can read and reason about the error.
pub fn discord_api_error(err: twilight_http::Error) -> Result<CallToolResult, rmcp::ErrorData> {
    let msg = format!("Discord API error: {err}");
    tracing::warn!("{msg}");
    Ok(CallToolResult::error(vec![Content::text(msg)]))
}

/// Convert a response deserialization error into an MCP tool result.
pub fn deserialize_error(
    err: twilight_http::response::DeserializeBodyError,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let msg = format!("Failed to deserialize Discord response: {err}");
    tracing::warn!("{msg}");
    Ok(CallToolResult::error(vec![Content::text(msg)]))
}

/// Create a successful tool result containing pretty-printed JSON.
pub fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(value).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("JSON serialization error: {e}"), None)
    })?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Create a successful tool result with a plain text message.
pub fn text_result(msg: impl Into<String>) -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}
