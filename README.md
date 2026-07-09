# discord-mcp-rs

A Rust [MCP](https://modelcontextprotocol.io/) server that exposes the **entire Discord REST API** as tools. Connect any MCP client (Claude Code, Claude Desktop, etc.) to interact with Discord — servers, channels, messages, members, roles, webhooks, applications, and more — scoped to whatever your bot token can do.

The tools are **generated dynamically from Discord's official [OpenAPI 3.1 specification](https://github.com/discord/discord-api-spec)**, so coverage tracks the real API rather than a hand-maintained subset.

## How it works

At startup the server parses Discord's OpenAPI spec (embedded in the binary) and turns **every operation into one MCP tool**:

- **Tool name = the operation's `operationId`** (e.g. `create_message`, `get_guild`, `ban_user_from_guild`).
- **Path and query parameters** are top-level tool arguments (e.g. `channel_id`, `limit`).
- **JSON request bodies** go in a single `body` argument (a JSON object matching Discord's schema, inlined into each tool's input schema).
- **File uploads** take **local filesystem paths** — a single `file` string, or a `files` array — which the server reads and sends as `multipart/form-data`.
- Each call is sent to Discord with `Authorization: Bot <token>`; **HTTP 429** responses are retried automatically using `Retry-After`.

Every tool returns the raw Discord response as:

```json
{ "status": 200, "ok": true, "body": { /* Discord's JSON, or null for 204 */ } }
```

Discord API errors (400/401/403/404/…) are returned as tool results with `"ok": false` (not as MCP protocol errors), so the model can read the error and adjust.

## Features

- **242 tools** — full coverage of the Discord **v10** REST API
- Generated from Discord's official OpenAPI spec (no hand-written wrappers)
- Rich, self-contained JSON Schemas per tool (request-body types inlined under `$defs`)
- File uploads by local path (attachments, avatars, sticker/soundboard assets)
- Automatic rate-limit (429) retries
- stdio transport — works with any MCP client
- Single self-contained binary; the spec is embedded at build time

### Coverage

All 242 Discord v10 operations (103 GET, 43 POST, 39 DELETE, 33 PATCH, 24 PUT), by resource area:

| Area | Tools | Examples |
|---|---|---|
| Guilds | 91 | guild CRUD, members, roles, bans, guild channels, emojis, stickers, scheduled events, auto-moderation, templates, widget, welcome screen, integrations, prune, vanity URL |
| Channels & Messages | 48 | channels, messages, reactions, threads, pins, permission overwrites, channel invites, polls, typing |
| Applications | 33 | global & guild application (slash) commands, entitlements, application emojis, role connection metadata, activity instances, attachment upload |
| Webhooks & Interactions | 15 | webhook CRUD, execute webhook, interaction responses & followups |
| Lobbies | 15 | game SDK lobby management |
| Users | 12 | current user, user lookup, DMs, connections, guild membership |
| Invites | 5 | get / delete invites |
| Partner / Embedded App SDK | 5 | partner-sdk endpoints |
| OAuth2 | 4 | authorization info, token introspection* |
| Stage Instances | 4 | create / get / update / delete |
| Misc | 6 | gateway, SKUs, sticker packs, standalone stickers, voice regions, default soundboard sounds |

\* OAuth2 / partner-SDK / lobby endpoints that require a **user (bearer) token** won't work with a bot token — see [Limitations](#limitations).

To see the exact tool list, use your MCP client's tool listing, or run the handshake in [Discovering tools](#discovering-tools).

## Prerequisites

- A Discord bot token — [create one here](https://discord.com/developers/applications).

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `DISCORD_TOKEN` | Yes | Bot token, sent as `Authorization: Bot <token>`. |
| `DISCORD_OPENAPI_PREVIEW` | No | If truthy (`1`/`true`/`yes`/`on`), use the embedded **preview** spec (unstable/experimental features) instead of the stable one. |
| `DISCORD_OPENAPI_SPEC` | No | Path to an external OpenAPI JSON file to load instead of the embedded spec (overrides the preview flag). Lets you swap in an updated spec without rebuilding. |

> There is no `DISCORD_APPLICATION_ID` variable. Tools that need an application ID take an `application_id` argument; get it from `get_my_application`, or use routes that accept the `@me` alias.

## Setup

### 1. Create a Discord bot

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications) and create an application.
2. Open **Bot → Reset Token** and copy the token.
3. Under **Privileged Gateway Intents**, enable **Server Members Intent** if you need to list guild members (`list_guild_members`).
4. Under **OAuth2 → URL Generator**, select the `bot` scope and the permissions you want, then use the generated URL to invite the bot to your server.

### 2. Install

#### Option A — download a prebuilt binary (recommended)

Grab the latest binary for your platform from [Releases](https://github.com/Craigeous/discord-mcp-rs/releases/latest):

| Platform | Binary |
|---|---|
| macOS (Apple Silicon) | `discord-mcp-rs-aarch64-apple-darwin` |
| macOS (Intel) | `discord-mcp-rs-x86_64-apple-darwin` |
| Linux (x86_64) | `discord-mcp-rs-x86_64-unknown-linux-gnu` |
| Linux (arm64) | `discord-mcp-rs-aarch64-unknown-linux-gnu` |
| Windows (x86_64) | `discord-mcp-rs-x86_64-pc-windows-msvc.exe` |

```bash
# Example: macOS Apple Silicon
curl -L -o discord-mcp-rs \
  https://github.com/Craigeous/discord-mcp-rs/releases/latest/download/discord-mcp-rs-aarch64-apple-darwin
chmod +x discord-mcp-rs
```

#### Option B — build from source

Requires a recent stable [Rust](https://rustup.rs/) toolchain (**1.94+**; the crate uses edition 2024).

```bash
git clone https://github.com/Craigeous/discord-mcp-rs.git
cd discord-mcp-rs
cargo build --release
```

The binary is written to `target/release/discord-mcp-rs`.

### 3. Configure your MCP client

> Replace `/path/to/discord-mcp-rs` with the actual path to your binary.

#### Claude Code

Using the CLI:

```bash
claude mcp add discord --env DISCORD_TOKEN=YOUR_TOKEN_HERE -- /path/to/discord-mcp-rs
```

Or add it to your Claude Code MCP settings (`~/.claude/settings.json` or a project `.claude/settings.json`):

```json
{
  "mcpServers": {
    "discord": {
      "command": "/path/to/discord-mcp-rs",
      "env": { "DISCORD_TOKEN": "YOUR_TOKEN_HERE" }
    }
  }
}
```

#### Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "discord": {
      "command": "/path/to/discord-mcp-rs",
      "env": { "DISCORD_TOKEN": "YOUR_TOKEN_HERE" }
    }
  }
}
```

To use the preview spec, add `"DISCORD_OPENAPI_PREVIEW": "1"` to the `env` block.

### 4. Verify

Ask your MCP client:

> "What Discord servers is the bot in?"

It should call `get_my_user` and `list_my_guilds` and return the bot's info and server list.

## Usage

Tools mirror the Discord API one-to-one. A few worked examples:

- **List a guild's channels** — `list_guild_channels` with `{ "guild_id": "..." }`
- **Send a message** — `create_message` with `{ "channel_id": "...", "body": { "content": "Hello!" } }`
- **Send a message with an attachment** — `create_message` with
  `{ "channel_id": "...", "body": { "content": "See attached" }, "files": ["/abs/path/image.png"] }`
- **Ban a member** — `ban_user_from_guild` with `{ "guild_id": "...", "user_id": "...", "body": { "delete_message_seconds": 0 } }`
- **Edit the bot's profile** — `update_my_user` with `{ "body": { "username": "New Name" } }`

### Tips

- **Snowflake IDs are strings** — pass IDs like `"1234567890123456789"`.
- **Request bodies go in `body`** — the shape is described by each tool's input schema (Discord's own field names).
- **File uploads are local paths** — the server reads the file itself; pass absolute paths where possible.
- **Permissions and intents matter** — if the bot lacks a permission (or a required privileged intent), the tool returns a Discord error with `"ok": false`; read it and adjust.
- **Application-scoped tools** — pass `application_id` (from `get_my_application`) where required.

### Discovering tools

To dump the full tool list over stdio:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | DISCORD_TOKEN="YOUR_TOKEN" ./target/release/discord-mcp-rs 2>/dev/null \
  | tail -1 | python3 -c 'import sys,json; print("\n".join(sorted(t["name"] for t in json.load(sys.stdin)["result"]["tools"])))'
```

## Limitations

- **Bot-token auth only.** Every request is sent with `Authorization: Bot <token>`. Endpoints that require an OAuth2 **user/bearer** token (parts of `oauth2`, `partner-sdk`, and `lobbies`) will fail with a bot token.
- **No audit-log reason header.** Discord's `X-Audit-Log-Reason` header isn't exposed (the spec models no header parameters).
- **Preview spec is unstable.** `DISCORD_OPENAPI_PREVIEW` exposes experimental operations that Discord may change or remove without notice.
- The spec is Discord's public preview and may differ from the [docs](https://discord.com/developers/docs) in places; when they disagree, follow the docs.

## Updating Discord API coverage

Coverage comes entirely from the vendored spec under `spec/` — there are no hand-written tools to touch. To update:

1. Re-vendor `spec/openapi.json` and `spec/openapi_preview.json` from [discord/discord-api-spec](https://github.com/discord/discord-api-spec) (record the upstream commit in `spec/SOURCE.txt`).
2. Rebuild. New/changed operations become tools automatically.

Pushing spec changes to `main` triggers the release workflow.

## Development

```bash
cargo test        # spec parsing + schema generation sanity checks (offline)
cargo clippy      # lints
cargo build --release
```

Project layout:

- `spec/` — vendored Discord OpenAPI specs (embedded via `include_str!`)
- `src/openapi.rs` — spec parser: builds the tool registry and per-tool JSON Schemas
- `src/server.rs` — `rmcp::ServerHandler` implementation: `list_tools` / `call_tool` dispatch
- `src/main.rs` — entry point, stderr logging, stdio transport

## Debug logging

```bash
RUST_LOG=debug DISCORD_TOKEN="YOUR_TOKEN" cargo run
```

Logs go to stderr so they don't interfere with the MCP stdio transport on stdout.

## License

MIT
