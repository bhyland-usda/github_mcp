use super::{GitHubServer, enc, normalize_state};
use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

// NOTE: No code is actually dead. The tool handler handles actually calling the tools and such the
// compiler cannot determine that it's not dead code and throws warnings.

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPullRequestParams {
    pub owner: String,
    pub repo: String,
    #[schemars(description = "Pull request number")]
    pub pull_number: u64,
}

#[tool_router(router = pull_requests_tool_router, vis = "pub(crate)")]
impl GitHubServer {
    #[tool(description = "Create a pull request for a GitHub repository")]
    pub async fn create_pull_request(
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
    pub async fn list_pull_requests(
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
    pub async fn get_pull_request(
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
