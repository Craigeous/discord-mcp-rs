# discord-mcp-rs

A Rust-based [MCP](https://modelcontextprotocol.io/) server that exposes the Discord REST API as tools. Connect any MCP client (Claude Code, Claude Desktop, etc.) to interact with Discord — managing servers, channels, messages, members, roles, and more — scoped to whatever permissions your bot token has.

## Features

- **152 tools** covering the Discord REST API
- Stdio transport (works with any MCP client)
- Automatic rate limiting via [twilight-http](https://github.com/twilight-rs/twilight)
- Discord API errors returned as tool results so the LLM can reason about them

### Tool Categories

| Category | Tools | Examples |
|---|---|---|
| Discovery | 3 | Get bot info, list guilds, guild details |
| Channels | 8 | List, create, update, delete, reorder, follow announcements, typing |
| Messages | 10 | Send, edit, delete, bulk delete, pin/unpin, crosspost |
| Reactions | 5 | Add, remove, list, clear reactions |
| Members | 5 | List, search, update, kick |
| Bans | 5 | List, get, ban, unban, bulk ban |
| Roles | 6 | CRUD, assign/remove from members |
| Threads | 10 | Create, join/leave, manage members, list active/archived |
| Webhooks | 7 | CRUD, execute |
| Invites | 5 | List, get, create, delete |
| Emojis | 9 | Guild & application emoji CRUD |
| Stickers | 5 | List, get, delete, sticker packs |
| Auto Moderation | 5 | CRUD rules |
| Scheduled Events | 6 | CRUD, list users |
| Audit Log | 1 | Query audit log with filters |
| Guild Settings | 11 | Update guild, prune, vanity URL, welcome screen, widget, voice regions, preview, create/delete guild, integrations |
| Permissions | 2 | Set/delete channel permission overwrites |
| Stage Instances | 4 | Create, get, update, delete stage instances |
| Voice | 3 | List regions, update voice states |
| Users | 4 | Get user, create DM, leave guild, connections |
| Application Commands | 8 | Global & guild slash command CRUD |
| Interactions | 8 | Respond to interactions, followup messages |
| Guild Templates | 6 | CRUD, sync templates |
| Polls | 2 | Get answer voters, end poll |
| Soundboard | 7 | Default & guild soundboard sounds CRUD, send sounds |
| Monetization | 7 | SKUs, entitlements, subscriptions |

## Prerequisites

- A Discord bot token ([create one here](https://discord.com/developers/applications))

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DISCORD_TOKEN` | Yes | Your Discord bot token |
| `DISCORD_APPLICATION_ID` | No | Application ID (auto-detected from bot user if not set) |

## Setup

### 1. Create a Discord Bot

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Go to **Bot** > **Reset Token** and copy your bot token
4. Under **Privileged Gateway Intents**, enable **Server Members Intent** (needed for member listing/searching)
5. Go to **OAuth2** > **URL Generator**, select the `bot` scope, choose your permissions, and use the generated URL to invite the bot to your server

### 2. Install

#### Option A: Download a prebuilt binary (recommended)

Download the latest binary for your platform from [Releases](https://github.com/Craigeous/discord-mcp-rs/releases/latest):

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

#### Option B: Build from source

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
git clone https://github.com/Craigeous/discord-mcp-rs.git
cd discord-mcp-rs
cargo build --release
```

The binary will be at `target/release/discord-mcp-rs`.

### 3. Configure your MCP client

> **Note:** Replace `/path/to/discord-mcp-rs` below with the actual path to your binary (downloaded or built).

#### Claude Code

Add to your Claude Code MCP settings (`~/.claude/settings.json` or project `.claude/settings.json`):

```json
{
  "mcpServers": {
    "discord": {
      "command": "/path/to/discord-mcp-rs",
      "env": {
        "DISCORD_TOKEN": "YOUR_TOKEN_HERE"
      }
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
      "env": {
        "DISCORD_TOKEN": "YOUR_TOKEN_HERE"
      }
    }
  }
}
```

### 4. Verify

Ask your MCP client:

> "What Discord servers am I in?"

It should call `get_current_user` and `list_guilds` and return your bot's info and server list.

## Usage Tips

- **Start with discovery**: `get_current_user` and `list_guilds` to see what the bot has access to
- **All IDs are strings**: Pass Discord snowflake IDs as strings (e.g. `"1234567890"`)
- **Permissions matter**: Tools will return Discord API errors if the bot lacks the required permissions — the LLM can read these and adjust
- **Audit log reasons**: Destructive tools like `kick_member` accept an optional `reason` parameter that appears in the guild's audit log

## Debug Logging

```bash
RUST_LOG=debug DISCORD_TOKEN="YOUR_TOKEN" cargo run
```

Logs go to stderr so they don't interfere with the MCP stdio transport.

## License

MIT
