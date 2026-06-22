use super::{GitHubServer, enc, normalize_state};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool,
    tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

// NOTE: No code is actually dead. The tool handler handles actually calling the tools and such the
// compiler cannot determine that it's not dead code and throws warnings.

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssuesParams {
    #[schemars(description = "Repository owner (username or organization)")]
    pub owner: String,
    #[schemars(description = "Respository name")]
    pub repo: String,
    #[schemars(description = "Issue state filter: open, closed, or all")]
    pub state: Option<String>,
    #[schemars(description = "Results per page (1-100, default 30")]
    pub per_page: Option<u8>,
    #[schemars(description = "Page number (default 1)")]
    pub page: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetIssueParams {
    #[schemars(description = "Repository owner (username or organization)")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Issue number")]
    pub issue_number: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueParams {
    #[schemars(description = "Repository owner (username or organization")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Issue title (required)")]
    pub title: String,
    #[schemars(description = "Issue body in markdown")]
    pub body: String,
    #[schemars(description = "Label names to attach")]
    pub labels: Option<Vec<String>>,
    #[schemars(description = "GitHub usernames to assign")]
    pub assignees: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssueLabelsParams {
    #[schemars(description = "Repository owner (username or organization)")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Issue number")]
    pub issue_number: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ModifyIssueLabelsParams {
    #[schemars(description = "Repository owner (username or organization")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Issue number")]
    pub issue_number: u32,
    #[schemars(
        description = "Labels to add to, replace, or remove from, depending on the tool call, existing labels"
    )]
    pub labels: Option<Vec<String>>,
}

#[tool_router(router = issues_tool_router, vis = "pub(crate)")]
impl GitHubServer {
    #[tool(description = "List issues for a GitHub repository")]
    pub async fn list_issues(
        &self,
        Parameters(params): Parameters<ListIssuesParams>,
    ) -> Result<CallToolResult, ErrorData> {
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

    #[tool(description = "Get a specific issue from a GitHub repository")]
    pub async fn get_issue(
        &self,
        Parameters(params): Parameters<GetIssueParams>,
    ) -> Result<CallToolResult, ErrorData> {
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
    pub async fn create_issue(
        &self,
        Parameters(params): Parameters<CreateIssueParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!("/repos/{}/{}/issues", enc(&params.owner), enc(&params.repo));

        let mut body = serde_json::Map::new();
        body.insert("title".to_string(), serde_json::Value::String(params.title));
        body.insert("body".to_string(), serde_json::Value::String(params.body));

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
    #[tool(description = "List the lables attached to a GitHub issue")]
    pub async fn list_issue_lables(
        &self,
        Parameters(params): Parameters<ListIssueLabelsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels",
            enc(&params.owner),
            enc(&params.repo),
            params.issue_number
        );

        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Replace labels on a GitHub issue")]
    pub async fn replace_issue_labels(
        &self,
        Parameters(params): Parameters<ModifyIssueLabelsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels",
            enc(&params.owner),
            enc(&params.repo),
            params.issue_number
        );

        let mut body = serde_json::Map::new();
        if let Some(labels) = params.labels {
            body.insert(
                "labels".to_string(),
                serde_json::Value::Array(
                    labels.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        let result = self
            .client
            .put(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Delete all labels for a GitHub issue")]
    pub async fn delete_issue_labels(
        &self,
        Parameters(params): Parameters<ModifyIssueLabelsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels",
            enc(&params.owner),
            enc(&params.repo),
            params.issue_number
        );

        let mut body = serde_json::Map::new();
        body.insert(
            "labels".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Null]),
        );

        let result = self
            .client
            .put(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Add one or more labels to the existing labels for a GitHub issue")]
    pub async fn add_issue_labels(
        &self,
        Parameters(params): Parameters<ModifyIssueLabelsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels",
            enc(&params.owner),
            enc(&params.repo),
            params.issue_number,
        );

        let mut body = serde_json::Map::new();
        if let Some(labels) = params.labels {
            body.insert(
                "labels".to_string(),
                serde_json::Value::Array(
                    labels.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        let result = self
            .client
            .post(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(result))
    }
}
