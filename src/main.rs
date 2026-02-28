use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

mod error;
mod server;
mod tools;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    // Logging must go to stderr — stdout is the MCP stdio transport
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Discord MCP server");

    let service = server::DiscordMcpServer::from_env()?;

    let server = service.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP serving error: {:?}", e);
    })?;

    server.waiting().await?;
    Ok(())
}
