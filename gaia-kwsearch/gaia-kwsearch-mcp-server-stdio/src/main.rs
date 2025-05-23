mod search;

use rmcp::{ServiceExt, transport::stdio};
use search::KeywordSearchServer;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_line_number(true)
        .init();

    tracing::info!("Starting Gaia Keyword Search MCP server in stdio mode");

    // Create an instance of our counter router
    let service = KeywordSearchServer.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
