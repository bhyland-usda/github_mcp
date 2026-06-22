mod github;
mod server;

use rmcp::{ServiceExt, transport::stdio};
use server::GitHubServer;
use std::io;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(io::stderr)
        .with_ansi(false)
        .init();

    let token = std::env::var("GITHUB_TOKEN").ok();
    if token.is_none() {
        tracing::warn!(
            "GITHUB_TOKEN not set; unauthenticated requests will have lower rate limits"
        );
    }

    let server = GitHubServer::new(token);
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("failed to start MCP server: {e:?}");
    })?;
    tracing::info!("GitHub MCP server running on stdio");
    service.waiting().await?;

    Ok(())
}
