use clap::{Parser, ValueEnum};
use rmcp::{model::CallToolRequestParam, service::ServiceExt, transport::TokioChildProcess};
use tokio::process::Command;
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

    // create a mcp client
    match cli.transport {
        TransportType::Tcp => {
            tracing::info!("Connecting to MCP server via tcp");

            // connect to mcp server
            let stream = tokio::net::TcpSocket::new_v4()?
                .connect(SOCKET_ADDR.parse()?)
                .await?;

            // create a mcp client
            let mcp_client = ().serve(stream).await?;

            // List available tools
            let tools = mcp_client.peer().list_tools(Default::default()).await?;
            tracing::info!("{}", serde_json::to_string_pretty(&tools)?);

            let request_param = CallToolRequestParam {
                name: "sum".into(),
                arguments: Some(serde_json::Map::from_iter([
                    ("a".to_string(), serde_json::Value::Number(1.into())),
                    ("b".to_string(), serde_json::Value::Number(2.into())),
                ])),
            };

            // Call the sum tool
            let sum_result = mcp_client.peer().call_tool(request_param).await?;

            tracing::info!("Sum result: {}", serde_json::to_string_pretty(&sum_result)?);
        }
        TransportType::Stdio => {
            tracing::info!("Connecting to MCP server via stdio");

            let transport = TokioChildProcess::new(&mut Command::new(
                "./target/release/gaia-calculator-mcp-server",
            ))?;

            let service = ().serve(transport).await?;

            // Initialize
            let server_info = service.peer_info();
            tracing::info!("Connected to server: {server_info:#?}");

            // List available tools
            let tools = service.peer().list_tools(Default::default()).await?;
            tracing::info!("{}", serde_json::to_string_pretty(&tools)?);

            let request_param = CallToolRequestParam {
                name: "sum".into(),
                arguments: Some(serde_json::Map::from_iter([
                    ("a".to_string(), serde_json::Value::Number(1.into())),
                    ("b".to_string(), serde_json::Value::Number(2.into())),
                ])),
            };

            // Call the sum tool
            let sum_result = service.peer().call_tool(request_param).await?;

            tracing::info!("Sum result: {}", serde_json::to_string_pretty(&sum_result)?);

            service.cancel().await?;
        }
    };

    Ok(())
}
