//! Dynamic MCP server: exposes every Discord OpenAPI operation as a tool and
//! dispatches calls straight to Discord's REST API.

use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
};
use serde_json::{Value, json};

use crate::openapi::{FileMode, Operation, Registry};

/// Embedded stable spec (the public, stable Discord API).
const SPEC_STABLE: &str = include_str!("../spec/openapi.json");
/// Embedded preview spec (unstable/experimental features).
const SPEC_PREVIEW: &str = include_str!("../spec/openapi_preview.json");

/// Max attempts when Discord returns HTTP 429 (rate limited).
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

/// Percent-encoding set for path parameters: encode everything except the
/// URL "unreserved" characters, so snowflakes and webhook/interaction tokens
/// (which contain `-`, `_`, `.`) pass through unescaped.
const PATH_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Clone)]
pub struct DiscordMcpServer {
    registry: Arc<Registry>,
    tools: Arc<Vec<rmcp::model::Tool>>,
    http: reqwest::Client,
    token: String,
}

impl DiscordMcpServer {
    /// Build the server from environment variables.
    ///
    /// * `DISCORD_TOKEN` (required) — bot token used for `Authorization: Bot`.
    /// * `DISCORD_OPENAPI_SPEC` (optional) — path to a spec file to load
    ///   instead of the embedded one.
    /// * `DISCORD_OPENAPI_PREVIEW` (optional) — if truthy, use the embedded
    ///   preview spec instead of the stable one.
    ///
    /// No network calls are made at startup.
    pub fn from_env() -> anyhow::Result<Self> {
        let token = std::env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable not set"))?;

        let spec = load_spec_text()?;
        let registry = Registry::from_spec_str(&spec)?;
        if registry.is_empty() {
            anyhow::bail!("spec produced no operations");
        }
        tracing::info!(
            "Loaded {} Discord API tools (base {})",
            registry.len(),
            registry.base_url
        );

        let tools = registry.tools();
        let http = reqwest::Client::builder()
            .user_agent(concat!("discord-mcp-rs/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            registry: Arc::new(registry),
            tools: Arc::new(tools),
            http,
            token,
        })
    }

    /// Execute one operation against the Discord API.
    async fn dispatch(&self, op: &Operation, args: &Value) -> anyhow::Result<CallToolResult> {
        // Resolve path parameters into the URL.
        let mut path = op.path_template.clone();
        for name in &op.path_params {
            let raw = args
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("missing required path parameter `{name}`"))?;
            let s = scalar_to_string(raw);
            let encoded = utf8_percent_encode(&s, PATH_SET).to_string();
            path = path.replace(&format!("{{{name}}}"), &encoded);
        }
        let url = format!("{}{}", self.registry.base_url, path);
        let method: reqwest::Method = op.method.parse()?;

        // Collect query parameters.
        let mut query: Vec<(String, String)> = Vec::new();
        for name in &op.query_params {
            if let Some(v) = args.get(name) {
                if v.is_null() {
                    continue;
                }
                push_query(&mut query, name, v);
            }
        }

        let body = args.get("body").filter(|v| !v.is_null());
        let use_multipart = self.wants_multipart(op, args);

        // Send, retrying on 429.
        let mut attempt = 0;
        let response = loop {
            let mut req = self
                .http
                .request(method.clone(), &url)
                .header("Authorization", format!("Bot {}", self.token))
                .query(&query);

            if use_multipart {
                req = req.multipart(self.build_multipart(op, args, body).await?);
            } else if let Some(b) = body {
                req = req.json(b);
            }

            let resp = req.send().await?;

            if resp.status().as_u16() == 429 && attempt < MAX_RATE_LIMIT_RETRIES {
                let wait = retry_after(&resp).unwrap_or(1.0).min(30.0);
                attempt += 1;
                tracing::warn!(
                    "rate limited on {} (attempt {attempt}); retrying in {wait:.2}s",
                    op.operation_id
                );
                tokio::time::sleep(Duration::from_secs_f64(wait)).await;
                continue;
            }
            break resp;
        };

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let body_json: Value = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };

        let payload = json!({
            "status": status.as_u16(),
            "ok": status.is_success(),
            "body": body_json,
        });
        let content = ContentBlock::json(&payload)
            .map_err(|e| anyhow::anyhow!("failed to encode response: {e}"))?;

        if status.is_success() {
            Ok(CallToolResult::success(vec![content]))
        } else {
            Ok(CallToolResult::error(vec![content]))
        }
    }

    /// Whether this call should be sent as multipart/form-data.
    fn wants_multipart(&self, op: &Operation, args: &Value) -> bool {
        match &op.file_mode {
            FileMode::None => false,
            FileMode::Named(fields) => fields
                .iter()
                .any(|f| args.get(f).and_then(|v| v.as_str()).is_some()),
            FileMode::Array => args
                .get("files")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().any(|v| v.is_string()))
                .unwrap_or(false),
        }
    }

    /// Build a multipart form with the JSON payload plus file parts.
    async fn build_multipart(
        &self,
        op: &Operation,
        args: &Value,
        body: Option<&Value>,
    ) -> anyhow::Result<reqwest::multipart::Form> {
        let mut form = reqwest::multipart::Form::new();

        // Discord expects the JSON body under the `payload_json` part.
        if let Some(b) = body {
            form = form.text("payload_json", serde_json::to_string(b)?);
        }

        match &op.file_mode {
            FileMode::Named(fields) => {
                for f in fields {
                    if let Some(path) = args.get(f).and_then(|v| v.as_str()) {
                        form = form.part(f.clone(), file_part(path).await?);
                    }
                }
            }
            FileMode::Array => {
                if let Some(arr) = args.get("files").and_then(|v| v.as_array()) {
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(path) = item.as_str() {
                            form = form.part(format!("files[{i}]"), file_part(path).await?);
                        }
                    }
                }
            }
            FileMode::None => {}
        }

        Ok(form)
    }
}

impl ServerHandler for DiscordMcpServer {
    fn get_info(&self) -> ServerInfo {
        // ServerInfo/Implementation are #[non_exhaustive]; build from Default.
        let mut info = ServerInfo::default();
        info.protocol_version = ProtocolVersion::LATEST;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info.name = "discord-mcp-rs".to_string();
        // Include the git commit as semver build metadata so a connected client
        // can tell exactly which build is running (all releases share 0.2.0).
        info.server_info.version = match env!("GIT_HASH") {
            "unknown" => env!("CARGO_PKG_VERSION").to_string(),
            git => format!("{}+{}", env!("CARGO_PKG_VERSION"), git),
        };
        info.instructions = Some(
            "Discord REST API as MCP tools, generated from Discord's official OpenAPI \
             spec. Each tool is named after its operationId. Pass path and query \
             parameters as top-level arguments; JSON request bodies go in the `body` \
             argument; file uploads take local filesystem paths (a `file` path or a \
             `files` array). Snowflake IDs are strings."
                .to_string(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tools.as_ref().clone()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.as_ref();
        let op = self.registry.get(name).ok_or_else(|| {
            McpError::invalid_params(format!("unknown tool: {name}"), None)
        })?;

        let args = request
            .arguments
            .map(Value::Object)
            .unwrap_or_else(|| Value::Object(Default::default()));

        match self.dispatch(op, &args).await {
            Ok(result) => Ok(result),
            // Surface transport/argument errors as tool errors, not protocol errors.
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "Discord request for `{name}` failed: {e}"
            ))])),
        }
    }
}

/// Select the spec text: explicit file path > preview flag > embedded stable.
fn load_spec_text() -> anyhow::Result<Cow<'static, str>> {
    if let Ok(path) = std::env::var("DISCORD_OPENAPI_SPEC") {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read DISCORD_OPENAPI_SPEC `{path}`: {e}"))?;
        tracing::info!("Loaded spec from {path}");
        return Ok(Cow::Owned(text));
    }
    let preview = std::env::var("DISCORD_OPENAPI_PREVIEW")
        .map(|v| is_truthy(&v))
        .unwrap_or(false);
    if preview {
        tracing::info!("Using embedded preview spec");
        Ok(Cow::Borrowed(SPEC_PREVIEW))
    } else {
        Ok(Cow::Borrowed(SPEC_STABLE))
    }
}

fn is_truthy(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

/// Render a scalar JSON value as a string for URL paths/queries.
fn scalar_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Append a query parameter, expanding arrays into repeated keys.
fn push_query(query: &mut Vec<(String, String)>, key: &str, v: &Value) {
    match v {
        Value::Array(items) => {
            for item in items {
                query.push((key.to_string(), scalar_to_string(item)));
            }
        }
        _ => query.push((key.to_string(), scalar_to_string(v))),
    }
}

/// Read the `Retry-After` header (seconds) from a 429 response.
fn retry_after(resp: &reqwest::Response) -> Option<f64> {
    resp.headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
}

/// Build a multipart file part by reading a local file.
async fn file_part(path: &str) -> anyhow::Result<reqwest::multipart::Part> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| anyhow::anyhow!("cannot read file `{path}`: {e}"))?;
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    Ok(reqwest::multipart::Part::bytes(bytes).file_name(filename))
}
