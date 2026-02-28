# Discord MCP Server

Rust-based MCP server providing Discord API tools via stdio transport.

## Build & Run
- `cargo build --release` to build
- `DISCORD_TOKEN="Bot YOUR_TOKEN" cargo run` to run
- `RUST_LOG=debug DISCORD_TOKEN="Bot YOUR_TOKEN" cargo run` for verbose logging

## Architecture
- `src/main.rs`: Entry point, stdio transport setup
- `src/server.rs`: DiscordMcpServer struct with all `#[tool]` methods (thin wrappers)
- `src/tools/`: One file per tool category, contains actual Discord API logic
- `src/error.rs`: Discord API error -> MCP result mapping
- `src/util.rs`: ID parsing helpers

## Conventions
- All tool parameters use String for Discord snowflake IDs
- Discord API errors are returned as tool results (not MCP protocol errors)
- Logging goes to stderr (stdout is the MCP stdio transport)
- Each tool method in server.rs delegates to tools/ module functions
- twilight-http handles rate limiting automatically
