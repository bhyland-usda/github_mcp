pub mod issues;
pub mod pull_requests;
pub mod repo;

use crate::github::GithubClient;
use rmcp::{ServerHandler, handler::server::router::tool::ToolRouter, tool_handler};
use std::sync::Arc;

// Code is not actually dead, the compiler just cannot determine where it is called
// due to the MCP crate creating it automatically.
#[allow(dead_code)]
#[derive(Clone)]
pub struct GitHubServer {
    client: Arc<GithubClient>,
}

impl GitHubServer {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Arc::new(GithubClient::new(token)),
        }
    }
}

// Code is not actually dead, the compiler just cannot determine where it is called since it's only
// used in submodules.
#[allow(dead_code)]
fn enc(text: &str) -> String {
    urlencoding::encode(text).to_string()
}

// Code is not actually dead, the compiler just cannot determine where it is called since it's only
// used in submodules.
#[allow(dead_code)]
fn normalize_state(state: Option<String>) -> String {
    match state.as_deref() {
        Some("closed") | Some("all") | Some("open") => state.unwrap().to_lowercase(),
        _ => "open".to_string(),
    }
}

impl GitHubServer {
    fn tool_router() -> ToolRouter<Self> {
        Self::repo_tool_router() + Self::issues_tool_router() + Self::pull_requests_tool_router()
    }
}

#[tool_handler(
    name = "github-mcp-server",
    version = "0.2.0",
    instructions = "MCP server for GitHub. Exposes tools to search repositories, read repository details, list/get issues, create issues, and list/get pull requests. Set GITHUB_TOKEN for authenticated requests."
)]
impl ServerHandler for GitHubServer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_mcp_tools_are_discoverable() {
        let tools = GitHubServer::tool_router().list_all();
        let tool_names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();

        assert_eq!(
            tool_names,
            vec![
                "add_issue_labels",
                "create_issue",
                "create_pull_request",
                "create_repository_labels",
                "delete_issue_labels",
                "delete_repository_label",
                "get_issue",
                "get_pull_request",
                "get_repository",
                "get_repository_labels",
                "list_issue_lables",
                "list_issues",
                "list_pull_requests",
                "replace_issue_labels",
                "search_repositories",
                "update_repository_label",
            ]
        );
    }
}
