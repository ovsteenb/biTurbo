// Standalone MCP server binary. Spawned by AI agents via stdio.
// Listens on stdin/stdout for JSON-RPC 2.0 (MCP protocol).

use anyhow::Result;
use biturbo_lib::mcp::run_mcp_server_stdio;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Force CPU-only ONNX Runtime before any library triggers ort init.
    std::env::set_var("ORT_DISABLE_CORE_ML", "1");
    std::env::set_var("ORT_DNNL_DISABLE", "1");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with_writer(std::io::stderr)
        .init();

    run_mcp_server_stdio().await
}
