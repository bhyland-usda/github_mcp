use super::{GitHubServer, enc};
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
pub struct SearchRespositoriesParams {
    #[schemars(description = "GitHub serach query, e.g. 'language:rust stars :> 1000'")]
    pub query: String,
    #[schemars(description = "Results per page(1-100, default 30")]
    pub per_page: Option<u8>,
    #[schemars(description = "Page number (default 1)")]
    pub page: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRepositoryParams {
    #[schemars(description = "Repository owner (user or organization)")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ModifyRepositoryLabelParams {
    #[schemars(description = "Respository owner (user or organization")]
    pub owner: String,
    #[schemars(description = "Repository name")]
    pub repo: String,
    #[schemars(description = "Label name to add to a specific GitHub repository")]
    pub name: String,
    #[schemars(description = "New label name, used when updating a specific label's name")]
    pub new_name: Option<String>,
    #[schemars(
        description = "New label's color within the repo (hexidecimal without the leading #)"
    )]
    pub color: Option<String>,
    #[schemars(
        description = "A short description of the new label. Must be 100 characters or fewer."
    )]
    pub description: Option<String>,
}

#[tool_router(router = repo_tool_router, vis = "pub(crate)")]
impl GitHubServer {
    #[tool(description = "Search for GitHub repositories using GitHub's search API")]
    pub async fn search_repositories(
        &self,
        Parameters(params): Parameters<SearchRespositoriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let per_page = params.per_page.unwrap_or(30).clamp(1, 100);
        let page = params.page.unwrap_or(1);
        let path = format!(
            "/search/repositories?q={}&per_page={per_page}&page={page}",
            enc(&params.query)
        );
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Get details for a specific GitHub repository")]
    pub async fn get_repository(
        &self,
        Parameters(params): Parameters<GetRepositoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!("/repos/{}/{}", enc(&params.owner), enc(&params.repo));
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "List labels for a specific GitHub repository")]
    async fn get_repository_labels(
        &self,
        Parameters(params): Parameters<GetRepositoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!("/repos/{}/{}/labels", enc(&params.owner), enc(&params.repo));
        let result = self.client.get(&path).await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Create labels for a specific GitHub repository")]
    pub async fn create_repository_labels(
        &self,
        Parameters(params): Parameters<ModifyRepositoryLabelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!("/repos/{}/{}/labels", enc(&params.owner), enc(&params.repo));

        let mut body = serde_json::Map::new();
        body.insert("name".to_string(), serde_json::Value::String(params.name));

        if let Some(color) = params.color {
            body.insert("color".to_string(), serde_json::Value::String(color));
        }

        if let Some(description) = params.description {
            body.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        let response = self
            .client
            .post(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(response))
    }

    #[tool(description = "Update a repository's name, color, and/or description")]
    pub async fn update_repository_label(
        &self,
        Parameters(params): Parameters<ModifyRepositoryLabelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!(
            "/repos/{}/{}/labels/{}",
            enc(&params.owner),
            enc(&params.repo),
            enc(&params.name)
        );
        let mut body = serde_json::Map::new();

        if let Some(new_name) = params.new_name {
            body.insert("new_name".to_string(), serde_json::Value::String(new_name));
        }

        if let Some(color) = params.color {
            body.insert("color".to_string(), serde_json::Value::String(color));
        }

        if let Some(description) = params.description {
            body.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        let response = self
            .client
            .patch(&path, serde_json::Value::Object(body))
            .await?;

        Ok(CallToolResult::structured(response))
    }

    #[tool(description = "Delete a specific label from a GitHub repository")]
    pub async fn delete_repository_label(
        &self,
        Parameters(params): Parameters<ModifyRepositoryLabelParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = format!(
            "/repos/{}/{}/labels/{}",
            enc(&params.owner),
            enc(&params.repo),
            enc(&params.name)
        );

        let response = self.client.delete(&path).await?;

        Ok(CallToolResult::structured(response))
    }
}
