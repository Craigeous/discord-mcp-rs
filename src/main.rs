use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

mod openapi;
mod server;

/// Provenance of the vendored spec (see `spec/SOURCE.txt`).
const SPEC_SOURCE: &str = include_str!("../spec/SOURCE.txt");

/// Full version string: crate version + build metadata (git commit, CI build number).
fn version_string() -> String {
    let mut s = format!("discord-mcp-rs {}", env!("CARGO_PKG_VERSION"));
    let git = env!("GIT_HASH");
    let build = option_env!("GITHUB_RUN_NUMBER");
    let mut tags = Vec::new();
    if git != "unknown" {
        tags.push(format!("git {git}"));
    }
    if let Some(b) = build {
        tags.push(format!("build {b}"));
    }
    if !tags.is_empty() {
        s.push_str(&format!(" ({})", tags.join(", ")));
    }
    s
}

/// The Discord spec commit the binary was built against, e.g.
/// `discord/discord-api-spec@d6f2c4208693`.
fn spec_source_line() -> String {
    let commit = SPEC_SOURCE
        .lines()
        .find_map(|l| l.strip_prefix("commit:"))
        .map(|c| c.trim())
        .map(|c| &c[..c.len().min(12)])
        .unwrap_or("unknown");
    format!("discord/discord-api-spec@{commit}")
}

fn print_help() {
    println!("{}", version_string());
    println!();
    println!("A Model Context Protocol (MCP) server for the Discord REST API,");
    println!("generated dynamically from Discord's official OpenAPI spec.");
    println!();
    println!("USAGE:");
    println!("    discord-mcp-rs            Run the MCP server over stdio");
    println!("    discord-mcp-rs --version  Print version and build info");
    println!("    discord-mcp-rs --help     Print this help");
    println!();
    println!("ENVIRONMENT:");
    println!("    DISCORD_TOKEN             Bot token (required to run the server)");
    println!("    DISCORD_OPENAPI_PREVIEW   Use the embedded preview spec if truthy");
    println!("    DISCORD_OPENAPI_SPEC      Path to an external OpenAPI spec to load");
    println!("    RUST_LOG                  Log filter (e.g. debug); logs go to stderr");
}

/// Handle `--version` / `--help` before starting the runtime.
/// Returns `true` if a CLI command was handled and the process should exit.
fn handle_cli() -> bool {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-V" | "version" => {
                println!("{}", version_string());
                println!("spec: {}", spec_source_line());
                return true;
            }
            "--help" | "-h" | "help" => {
                print_help();
                return true;
            }
            _ => {}
        }
    }
    false
}

fn main() -> Result<()> {
    if handle_cli() {
        return Ok(());
    }

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Logging must go to stderr — stdout is the MCP stdio transport.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting {} ({})", version_string(), spec_source_line());

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let service = server::DiscordMcpServer::from_env()?;
        let server = service.serve(stdio()).await.inspect_err(|e| {
            tracing::error!("MCP serving error: {:?}", e);
        })?;
        server.waiting().await?;
        Ok(())
    })
}
