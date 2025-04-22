mod calculator;

use calculator::Calculator;
use clap::{Parser, ValueEnum};
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const SOCKET_ADDR: &str = "127.0.0.1:8001";

#[derive(Debug, Clone, ValueEnum)]
enum TransportType {
    Tcp,
    Stdio,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Calculator MCP server")]
struct Args {
    /// Transport type to use (tcp or stdio)
    #[arg(short, long, value_enum, default_value = "tcp")]
    transport: TransportType,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("info,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();

    let cli = Args::parse();

    match cli.transport {
        TransportType::Stdio => {
            // Create an instance of our counter router
            let service = Calculator.serve(stdio()).await.inspect_err(|e| {
                tracing::error!("serving error: {:?}", e);
            })?;

            service.waiting().await?;
        }
        TransportType::Tcp => {
            let tcp_listener = tokio::net::TcpListener::bind(SOCKET_ADDR).await?;
            tracing::info!("Calculator MCP server is listening on {}", SOCKET_ADDR);

            while let Ok((stream, _socket_addr)) = tcp_listener.accept().await {
                // spawn a new task to handle the connection
                tokio::spawn(async move {
                    // create a mcp server
                    let mcp_server = Calculator.serve(stream).await?;

                    // wait for the connection to be closed
                    mcp_server.waiting().await?;

                    anyhow::Ok(())
                });
            }
        }
    }

    Ok(())
}
