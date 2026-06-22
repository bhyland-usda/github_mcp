use crate::github::GithubClient;
use rmcp::{
    ServerHandler, handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool,
    tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct GithubServer {
    client: Arc<GithubClient>,
}

impl GithubServer {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Arc::new(GithubClient::new(token)),
        }
    }
}

fn enc(text: &str) -> String {
    urlencoding::encode(text).to_string()
}

fn normalize_state(state: Option<String>) -> String {
    match state.as_deref() {
        Some("closed") | Some("all") | Some("open") => state.unwrap().to_lowercase(),
        _ => "open".to_string(),
    }
}

// Tools
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRepositoriesParams {
    #[schemars(description = "GitHub search query, e.g. 'language:rust stars :>1000'")]
    pub query: String,
    #[schemars(description = "Results per page (1-100, default 30")]
    pub per_page: Option<u8>,
    #[schemars(description = "Page number (default 1)")]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRepositoryParams {
    #[schemars(description = "Repository owner (user or organization")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesParams {
    #[schemars(description = "Repository owner")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Issue state filter: open, closed, or all")]
    pub state: Option<String>,
    #[schemars(description = "Results per page (1-100, default 30")]
    pub per_page: Option<u8>,
    #[schemars(description = "Page number (default 1)")]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssuesParams {
    pub owner: String,
    pub repo: String,
    #[schemars(description = "Issue number")]
    pub issue_number: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueParams {
    pub owner: String,
    pub repo: String,
    pub title: String,
    #[schemars(description = "Issue body in markdown")]
    pub body: Option<String>,
    #[schemars(description = "Label names to attach")]
    pub labels: Option<Vec<String>>,
    #[schemars(description = "GitHub usernames to assign")]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePullRequestParams {
    pub owner: String,
    pub repo: String,
    #[schemars(description = "Pull request title")]
    pub title: String,
    #[schemars(description = "Pull request body")]
    pub body: String,
    #[schemars(
        description = "The repository and branch or this repository's branch that the feature is coming from (fork:new-feature)"
    )]
    pub head: String,
    #[schemars(description = "The branch the PR is meant to be merged into")]
    pub base: String,
    #[schemars(description = "An optional issue to link the PR to")]
    pub issue: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPullRequestsParams {
    pub owner: String,
    pub repo: String,
    #[schemars(description = "Pull request state filter: open, closed, or all")]
    pub state: Option<String>,
    #[schemars(description = "Results per page (1-100, default 30")]
    pub per_page: Option<u8>,
    #[schemars(description = "Page number (default 1")]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPullRequestParams {
    pub owner: String,
    pub repo: String,
    #[schemars(description = "Pull request number")]
    pub pull_number: u64,
}

// Tool router
#[tool_router]
impl GithubServer {
    #[tool(description = "Search for GitHub repositories using GitHub's search API")]
    async fn search_repositories(
        &self,
        Parameters(params): Parameters<SearchRepositoriesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let per_page = params.per_page.unwrap_or(30).clamp(1, 100);
        let page = params.page.unwrap_or(1);
        let path = format!(
            "/search/repositories?q={}&per_page={per_page}&page={page}",
            enc(&params.query),
        );
        let result = self.client.get(&path).await?;
        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Get details for a specific GitHub repository")]
    async fn get_repository(
        &self,
        Parameters(params): Parameters<GetRepositoryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!("/repos/{}/{}", enc(&params.owner), enc(&params.repo));
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "List issues for a GitHub repository")]
    async fn list_issues(
        &self,
        Parameters(params): Parameters<ListIssuesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let per_page = params.per_page.unwrap_or(30).clamp(1, 100);
        let page = params.page.unwrap_or(1);
        let state = normalize_state(params.state);
        let path = format!(
            "/repos/{}/{}/issues?state={}&per_page={per_page}&page={page}",
            enc(&params.owner),
            enc(&params.repo),
            enc(&state)
        );
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Get a specific issue from a repository")]
    async fn get_issue(
        &self,
        Parameters(params): Parameters<GetIssuesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/issues/{}",
            enc(&params.owner),
            enc(&params.repo),
            params.issue_number
        );
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Create a new issue in a repository")]
    async fn create_issue(
        &self,
        Parameters(params): Parameters<CreateIssueParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!("/repos/{}/{}/issues", enc(&params.owner), enc(&params.repo),);

        let mut body = serde_json::Map::new();
        body.insert("title".to_string(), serde_json::Value::String(params.title));

        if let Some(bod) = params.body {
            body.insert("body".to_string(), serde_json::Value::String(bod));
        }

        if let Some(labels) = params.labels {
            body.insert(
                "labels".to_string(),
                serde_json::Value::Array(
                    labels.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        if let Some(assignees) = params.assignees {
            body.insert(
                "assignees".to_string(),
                serde_json::Value::Array(
                    assignees
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }

        let result = self
            .client
            .post(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Create a pull request for a GitHub repository")]
    async fn create_pull_request(
        &self,
        Parameters(params): Parameters<CreatePullRequestParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!("/repos/{}/{}/pulls", enc(&params.owner), enc(&params.repo));

        let mut body = serde_json::Map::new();
        body.insert("title".to_string(), serde_json::Value::String(params.title));
        body.insert("body".to_string(), serde_json::Value::String(params.body));
        body.insert("head".to_string(), serde_json::Value::String(params.head));
        body.insert("base".to_string(), serde_json::Value::String(params.base));
        body.insert(
            "maintainer_can_modify".to_string(),
            serde_json::Value::Bool(true),
        );

        if let Some(issue) = params.issue {
            body.insert("issue".to_string(), serde_json::Value::String(issue));
        }

        let result = self
            .client
            .post(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "List pull requests for a GitHub repository")]
    async fn list_pull_requests(
        &self,
        Parameters(params): Parameters<ListPullRequestsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let per_page = params.per_page.unwrap_or(30).clamp(1, 100);
        let page = params.page.unwrap_or(1);
        let state = normalize_state(params.state);
        let path = format!(
            "/repos/{}/{}/pulls?state={}&per_page={per_page}&page={page}",
            enc(&params.owner),
            enc(&params.repo),
            enc(&state)
        );
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Get a specific pull request from a repository")]
    async fn get_pull_request(
        &self,
        Parameters(params): Parameters<GetPullRequestParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/pulls/{}",
            enc(&params.owner),
            enc(&params.repo),
            params.pull_number
        );
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }
}

#[tool_handler(
    name = "github-mcp-server",
    version = "0.1.1",
    instructions = "MCP server for GitHub. Exposes tools to search repositories, read repository details, list/get issues, create issues, and list/get pull requests. Set GITHUB_TOKEN for authenticated requests."
)]
impl ServerHandler for GithubServer {}
