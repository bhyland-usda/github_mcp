use super::{GitHubServer, enc};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, schemars, tool,
    tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

const PROJECT_FIELDS_QUERY: &str = r#"
query($projectId: ID!) {
  node(id: $projectId) {
    ... on ProjectV2 {
      id
      title
      url
      fields(first: 100) {
        nodes {
          ... on ProjectV2FieldCommon {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
            dataType
            configuration {
              iterations {
                id
                title
                startDate
                duration
              }
              completedIterations {
                id
                title
                startDate
                duration
              }
            }
          }
        }
      }
    }
  }
}
"#;

const PROJECT_ITEMS_QUERY: &str = r#"
query($projectId: ID!, $first: Int!, $after: String) {
  node(id: $projectId) {
    ... on ProjectV2 {
      id
      title
      url
      items(first: $first, after: $after) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          id
          type
          isArchived
          content {
            ... on Issue {
              id
              number
              title
              url
              state
              repository {
                name
                owner {
                  login
                }
              }
            }
            ... on PullRequest {
              id
              number
              title
              url
              state
              repository {
                name
                owner {
                  login
                }
              }
            }
            ... on DraftIssue {
              id
              title
              body
            }
          }
          fieldValues(first: 100) {
            nodes {
              ... on ProjectV2ItemFieldTextValue {
                text
                field {
                  ... on ProjectV2FieldCommon {
                    id
                    name
                    dataType
                  }
                }
              }
              ... on ProjectV2ItemFieldNumberValue {
                number
                field {
                  ... on ProjectV2FieldCommon {
                    id
                    name
                    dataType
                  }
                }
              }
              ... on ProjectV2ItemFieldDateValue {
                date
                field {
                  ... on ProjectV2FieldCommon {
                    id
                    name
                    dataType
                  }
                }
              }
              ... on ProjectV2ItemFieldSingleSelectValue {
                name
                optionId
                field {
                  ... on ProjectV2FieldCommon {
                    id
                    name
                    dataType
                  }
                }
              }
              ... on ProjectV2ItemFieldIterationValue {
                title
                iterationId
                startDate
                duration
                field {
                  ... on ProjectV2FieldCommon {
                    id
                    name
                    dataType
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProjectParams {
    #[schemars(description = "Project owner type: organization, user, or viewer")]
    pub owner_type: String,
    #[schemars(description = "Organization or user login. Not required when owner_type is viewer")]
    pub owner_login: Option<String>,
    #[schemars(description = "ProjectV2 number")]
    pub project_number: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectIdParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListProjectItemsParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(description = "Results per page (1-100, default 30)")]
    pub first: Option<u8>,
    #[schemars(description = "Optional GraphQL pagination cursor")]
    pub after: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddIssueToProjectParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(
        description = "Issue node ID. If omitted, owner, repo, and issue_number are required"
    )]
    pub issue_node_id: Option<String>,
    #[schemars(description = "Repository owner for looking up the issue node ID")]
    pub owner: Option<String>,
    #[schemars(description = "Repository name for looking up the issue node ID")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number for looking up the issue node ID")]
    pub issue_number: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectIssueParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(
        description = "ProjectV2 item node ID. If omitted, owner, repo, and issue_number are required"
    )]
    pub item_id: Option<String>,
    #[schemars(description = "Repository owner for finding the project item")]
    pub owner: Option<String>,
    #[schemars(description = "Repository name for finding the project item")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number for finding the project item")]
    pub issue_number: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetProjectItemFieldParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(
        description = "ProjectV2 item node ID. If omitted, owner, repo, and issue_number are required"
    )]
    pub item_id: Option<String>,
    #[schemars(description = "Repository owner for finding the project item")]
    pub owner: Option<String>,
    #[schemars(description = "Repository name for finding the project item")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number for finding the project item")]
    pub issue_number: Option<u64>,
    #[schemars(description = "Project field node ID. If omitted, field_name is required")]
    pub field_id: Option<String>,
    #[schemars(
        description = "Project field name, such as Status, Priority, Estimate, Start date, or Iteration"
    )]
    pub field_name: Option<String>,
    #[schemars(description = "Field value type: text, number, date, single_select, or iteration")]
    pub value_type: String,
    #[schemars(
        description = "Value to set. For number, provide a numeric string. For date, use YYYY-MM-DD"
    )]
    pub value: String,
    #[schemars(
        description = "Single-select option ID. If omitted for single_select, value is matched to an option name"
    )]
    pub option_id: Option<String>,
    #[schemars(
        description = "Iteration ID. If omitted for iteration, value is matched to an iteration title"
    )]
    pub iteration_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClearProjectItemFieldParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(
        description = "ProjectV2 item node ID. If omitted, owner, repo, and issue_number are required"
    )]
    pub item_id: Option<String>,
    #[schemars(description = "Repository owner for finding the project item")]
    pub owner: Option<String>,
    #[schemars(description = "Repository name for finding the project item")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number for finding the project item")]
    pub issue_number: Option<u64>,
    #[schemars(description = "Project field node ID. If omitted, field_name is required")]
    pub field_id: Option<String>,
    #[schemars(description = "Project field name")]
    pub field_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveProjectIssueParams {
    #[schemars(description = "ProjectV2 node ID")]
    pub project_id: String,
    #[schemars(
        description = "ProjectV2 item node ID. If omitted, owner, repo, and issue_number are required"
    )]
    pub item_id: Option<String>,
    #[schemars(description = "Repository owner for finding the project item")]
    pub owner: Option<String>,
    #[schemars(description = "Repository name for finding the project item")]
    pub repo: Option<String>,
    #[schemars(description = "Issue number for finding the project item")]
    pub issue_number: Option<u64>,
    #[schemars(description = "Status field node ID. If omitted, field_name is used")]
    pub field_id: Option<String>,
    #[schemars(description = "Kanban/status field name. Defaults to Status")]
    pub field_name: Option<String>,
    #[schemars(description = "Target kanban column/status option name")]
    pub column_name: String,
    #[schemars(
        description = "Target single-select option ID. If omitted, column_name is matched to an option"
    )]
    pub option_id: Option<String>,
}

#[tool_router(router = projects_tool_router, vis = "pub(crate)")]
impl GitHubServer {
    #[tool(description = "Get a GitHub Projects v2 project with its fields and selectable options")]
    pub async fn get_project(
        &self,
        Parameters(params): Parameters<GetProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let owner_type = params.owner_type.to_lowercase();

        let (query, variables) = match owner_type.as_str() {
            "organization" | "org" => {
                let login = require_param(
                    params.owner_login,
                    "owner_login is required for organization projects",
                )?;
                (
                    r#"
query($login: String!, $number: Int!) {
  organization(login: $login) {
    projectV2(number: $number) {
      id
      title
      shortDescription
      public
      closed
      url
      fields(first: 100) {
        nodes {
          ... on ProjectV2FieldCommon {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
            dataType
            configuration {
              iterations {
                id
                title
                startDate
                duration
              }
              completedIterations {
                id
                title
                startDate
                duration
              }
            }
          }
        }
      }
    }
  }
}
"#,
                    json!({ "login": login, "number": params.project_number }),
                )
            }
            "user" => {
                let login = require_param(
                    params.owner_login,
                    "owner_login is required for user projects",
                )?;
                (
                    r#"
query($login: String!, $number: Int!) {
  user(login: $login) {
    projectV2(number: $number) {
      id
      title
      shortDescription
      public
      closed
      url
      fields(first: 100) {
        nodes {
          ... on ProjectV2FieldCommon {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
            dataType
            configuration {
              iterations {
                id
                title
                startDate
                duration
              }
              completedIterations {
                id
                title
                startDate
                duration
              }
            }
          }
        }
      }
    }
  }
}
"#,
                    json!({ "login": login, "number": params.project_number }),
                )
            }
            "viewer" | "me" => (
                r#"
query($number: Int!) {
  viewer {
    projectV2(number: $number) {
      id
      title
      shortDescription
      public
      closed
      url
      fields(first: 100) {
        nodes {
          ... on ProjectV2FieldCommon {
            id
            name
            dataType
          }
          ... on ProjectV2SingleSelectField {
            id
            name
            dataType
            options {
              id
              name
            }
          }
          ... on ProjectV2IterationField {
            id
            name
            dataType
            configuration {
              iterations {
                id
                title
                startDate
                duration
              }
              completedIterations {
                id
                title
                startDate
                duration
              }
            }
          }
        }
      }
    }
  }
}
"#,
                json!({ "number": params.project_number }),
            ),
            _ => {
                return Err(invalid_request(
                    "owner_type must be organization, user, or viewer",
                ));
            }
        };

        let result = self.client.graphql(query, variables).await?;
        Ok(CallToolResult::structured(result))
    }

    #[tool(
        description = "Get fields, field IDs, and selectable options for a GitHub Projects v2 project"
    )]
    pub async fn get_project_fields(
        &self,
        Parameters(params): Parameters<ProjectIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let result = self
            .client
            .graphql(
                PROJECT_FIELDS_QUERY,
                json!({ "projectId": params.project_id }),
            )
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(
        description = "List items in a GitHub Projects v2 project, including issue content and field values"
    )]
    pub async fn list_project_items(
        &self,
        Parameters(params): Parameters<ListProjectItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let first = params.first.unwrap_or(30).clamp(1, 100);

        let result = self
            .client
            .graphql(
                PROJECT_ITEMS_QUERY,
                json!({
                    "projectId": params.project_id,
                    "first": first,
                    "after": params.after,
                }),
            )
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Add an issue to a GitHub Projects v2 project")]
    pub async fn add_issue_to_project(
        &self,
        Parameters(params): Parameters<AddIssueToProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_node_id = match params.issue_node_id {
            Some(issue_node_id) => issue_node_id,
            None => {
                let owner = require_param(
                    params.owner,
                    "owner is required when issue_node_id is omitted",
                )?;
                let repo = require_param(
                    params.repo,
                    "repo is required when issue_node_id is omitted",
                )?;
                let issue_number = params.issue_number.ok_or_else(|| {
                    invalid_request("issue_number is required when issue_node_id is omitted")
                })?;

                get_issue_node_id(self, &owner, &repo, issue_number).await?
            }
        };

        let query = r#"
mutation($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
    item {
      id
      type
      content {
        ... on Issue {
          id
          number
          title
          url
          repository {
            name
            owner {
              login
            }
          }
        }
      }
    }
  }
}
"#;

        let result = self
            .client
            .graphql(
                query,
                json!({
                    "projectId": params.project_id,
                    "contentId": issue_node_id,
                }),
            )
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Find the GitHub Projects v2 item for an issue")]
    pub async fn get_project_item_for_issue(
        &self,
        Parameters(params): Parameters<ProjectIssueParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let owner = require_param(params.owner, "owner is required")?;
        let repo = require_param(params.repo, "repo is required")?;
        let issue_number = params
            .issue_number
            .ok_or_else(|| invalid_request("issue_number is required"))?;

        let item =
            find_project_item_for_issue(self, &params.project_id, &owner, &repo, issue_number)
                .await?;
        Ok(CallToolResult::structured(item))
    }

    #[tool(description = "Remove an issue item from a GitHub Projects v2 project")]
    pub async fn remove_issue_from_project(
        &self,
        Parameters(params): Parameters<ProjectIssueParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let item_id = resolve_item_id(
            self,
            &params.project_id,
            params.item_id,
            params.owner,
            params.repo,
            params.issue_number,
        )
        .await?;

        let query = r#"
mutation($projectId: ID!, $itemId: ID!) {
  deleteProjectV2Item(input: { projectId: $projectId, itemId: $itemId }) {
    deletedItemId
  }
}
"#;

        let result = self
            .client
            .graphql(
                query,
                json!({
                    "projectId": params.project_id,
                    "itemId": item_id,
                }),
            )
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(
        description = "Fill out or update a field value for an issue in a GitHub Projects v2 project"
    )]
    pub async fn set_project_item_field(
        &self,
        Parameters(params): Parameters<SetProjectItemFieldParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let item_id = resolve_item_id(
            self,
            &params.project_id,
            params.item_id,
            params.owner,
            params.repo,
            params.issue_number,
        )
        .await?;

        let field_id = resolve_field_id(
            self,
            &params.project_id,
            params.field_id,
            params.field_name.as_deref(),
        )
        .await?;

        let value_type = params.value_type.to_lowercase();
        let field_value = match value_type.as_str() {
            "text" => json!({ "text": params.value }),
            "number" => {
                let number = params.value.parse::<f64>().map_err(|_| {
                    invalid_request("value must be a valid number when value_type is number")
                })?;
                json!({ "number": number })
            }
            "date" => json!({ "date": params.value }),
            "single_select" | "single-select" | "singleselect" => {
                let option_id = match params.option_id {
                    Some(option_id) => option_id,
                    None => {
                        resolve_single_select_option_id(
                            self,
                            &params.project_id,
                            &field_id,
                            &params.value,
                        )
                        .await?
                    }
                };
                json!({ "singleSelectOptionId": option_id })
            }
            "iteration" => {
                let iteration_id = match params.iteration_id {
                    Some(iteration_id) => iteration_id,
                    None => {
                        resolve_iteration_id(self, &params.project_id, &field_id, &params.value)
                            .await?
                    }
                };
                json!({ "iterationId": iteration_id })
            }
            _ => {
                return Err(invalid_request(
                    "value_type must be text, number, date, single_select, or iteration",
                ));
            }
        };

        let result =
            update_project_field(self, &params.project_id, &item_id, &field_id, field_value)
                .await?;
        Ok(CallToolResult::structured(result))
    }

    #[tool(description = "Clear a field value for an issue in a GitHub Projects v2 project")]
    pub async fn clear_project_item_field(
        &self,
        Parameters(params): Parameters<ClearProjectItemFieldParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let item_id = resolve_item_id(
            self,
            &params.project_id,
            params.item_id,
            params.owner,
            params.repo,
            params.issue_number,
        )
        .await?;

        let field_id = resolve_field_id(
            self,
            &params.project_id,
            params.field_id,
            params.field_name.as_deref(),
        )
        .await?;

        let query = r#"
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!) {
  clearProjectV2ItemFieldValue(input: { projectId: $projectId, itemId: $itemId, fieldId: $fieldId }) {
    projectV2Item {
      id
    }
  }
}
"#;

        let result = self
            .client
            .graphql(
                query,
                json!({
                    "projectId": params.project_id,
                    "itemId": item_id,
                    "fieldId": field_id,
                }),
            )
            .await?;

        Ok(CallToolResult::structured(result))
    }

    #[tool(
        description = "Move an issue between kanban/status columns in a GitHub Projects v2 project"
    )]
    pub async fn move_project_issue_to_column(
        &self,
        Parameters(params): Parameters<MoveProjectIssueParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let item_id = resolve_item_id(
            self,
            &params.project_id,
            params.item_id,
            params.owner,
            params.repo,
            params.issue_number,
        )
        .await?;

        let field_name = params.field_name.as_deref().unwrap_or("Status");
        let field_id =
            resolve_field_id(self, &params.project_id, params.field_id, Some(field_name)).await?;

        let option_id = match params.option_id {
            Some(option_id) => option_id,
            None => {
                resolve_single_select_option_id(
                    self,
                    &params.project_id,
                    &field_id,
                    &params.column_name,
                )
                .await?
            }
        };

        let result = update_project_field(
            self,
            &params.project_id,
            &item_id,
            &field_id,
            json!({ "singleSelectOptionId": option_id }),
        )
        .await?;

        Ok(CallToolResult::structured(result))
    }
}

async fn get_issue_node_id(
    server: &GitHubServer,
    owner: &str,
    repo: &str,
    issue_number: u64,
) -> Result<String, ErrorData> {
    let path = format!(
        "/repos/{}/{}/issues/{}",
        enc(owner),
        enc(repo),
        issue_number
    );

    let issue = server.client.get(&path).await?;
    issue
        .get("node_id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| invalid_request("GitHub issue response did not include node_id"))
}

async fn resolve_item_id(
    server: &GitHubServer,
    project_id: &str,
    item_id: Option<String>,
    owner: Option<String>,
    repo: Option<String>,
    issue_number: Option<u64>,
) -> Result<String, ErrorData> {
    if let Some(item_id) = item_id {
        return Ok(item_id);
    }

    let owner = require_param(owner, "owner is required when item_id is omitted")?;
    let repo = require_param(repo, "repo is required when item_id is omitted")?;
    let issue_number = issue_number
        .ok_or_else(|| invalid_request("issue_number is required when item_id is omitted"))?;

    let item = find_project_item_for_issue(server, project_id, &owner, &repo, issue_number).await?;

    item.get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| invalid_request("Project item response did not include id"))
}

async fn find_project_item_for_issue(
    server: &GitHubServer,
    project_id: &str,
    owner: &str,
    repo: &str,
    issue_number: u64,
) -> Result<Value, ErrorData> {
    let owner_lower = owner.to_lowercase();
    let repo_lower = repo.to_lowercase();
    let mut after: Option<String> = None;

    for _ in 0..10 {
        let result = server
            .client
            .graphql(
                PROJECT_ITEMS_QUERY,
                json!({
                    "projectId": project_id,
                    "first": 100,
                    "after": after,
                }),
            )
            .await?;

        let items = result
            .pointer("/data/node/items/nodes")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid_request("Project items response was missing items"))?;

        for item in items {
            let content = match item.get("content") {
                Some(content) if !content.is_null() => content,
                _ => continue,
            };

            let item_number = content.get("number").and_then(Value::as_u64);
            let item_repo = content
                .pointer("/repository/name")
                .and_then(Value::as_str)
                .map(str::to_lowercase);
            let item_owner = content
                .pointer("/repository/owner/login")
                .and_then(Value::as_str)
                .map(str::to_lowercase);

            if item_number == Some(issue_number)
                && item_repo.as_deref() == Some(repo_lower.as_str())
                && item_owner.as_deref() == Some(owner_lower.as_str())
            {
                return Ok(item.clone());
            }
        }

        let page_info = result
            .pointer("/data/node/items/pageInfo")
            .ok_or_else(|| invalid_request("Project items response was missing pageInfo"))?;

        let has_next_page = page_info
            .get("hasNextPage")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if !has_next_page {
            break;
        }

        after = page_info
            .get("endCursor")
            .and_then(Value::as_str)
            .map(ToString::to_string);
    }

    Err(invalid_request(
        "Issue was not found in the project. Add the issue to the project first, or pass item_id explicitly",
    ))
}

async fn resolve_field_id(
    server: &GitHubServer,
    project_id: &str,
    field_id: Option<String>,
    field_name: Option<&str>,
) -> Result<String, ErrorData> {
    if let Some(field_id) = field_id {
        return Ok(field_id);
    }

    let field_name = field_name
        .ok_or_else(|| invalid_request("field_name is required when field_id is omitted"))?;

    let field = find_project_field(server, project_id, field_name).await?;
    field
        .get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| invalid_request("Project field did not include id"))
}

async fn find_project_field(
    server: &GitHubServer,
    project_id: &str,
    field_name: &str,
) -> Result<Value, ErrorData> {
    let fields = get_project_fields(server, project_id).await?;
    let field_name_lower = field_name.to_lowercase();

    fields
        .into_iter()
        .find(|field| {
            field
                .get("name")
                .and_then(Value::as_str)
                .map(|name| name.to_lowercase() == field_name_lower)
                .unwrap_or(false)
        })
        .ok_or_else(|| invalid_request(&format!("Project field '{field_name}' was not found")))
}

async fn get_project_fields(
    server: &GitHubServer,
    project_id: &str,
) -> Result<Vec<Value>, ErrorData> {
    let result = server
        .client
        .graphql(PROJECT_FIELDS_QUERY, json!({ "projectId": project_id }))
        .await?;

    result
        .pointer("/data/node/fields/nodes")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| invalid_request("Project fields response was missing fields"))
}

async fn resolve_single_select_option_id(
    server: &GitHubServer,
    project_id: &str,
    field_id: &str,
    option_name: &str,
) -> Result<String, ErrorData> {
    let fields = get_project_fields(server, project_id).await?;
    let option_name_lower = option_name.to_lowercase();

    let field = fields
        .iter()
        .find(|field| field.get("id").and_then(Value::as_str) == Some(field_id))
        .ok_or_else(|| invalid_request("Project field was not found"))?;

    let options = field
        .get("options")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid_request("Project field does not have single-select options"))?;

    options
        .iter()
        .find(|option| {
            option
                .get("name")
                .and_then(Value::as_str)
                .map(|name| name.to_lowercase() == option_name_lower)
                .unwrap_or(false)
        })
        .and_then(|option| option.get("id").and_then(Value::as_str))
        .map(ToString::to_string)
        .ok_or_else(|| {
            invalid_request(&format!(
                "Single-select option '{option_name}' was not found"
            ))
        })
}

async fn resolve_iteration_id(
    server: &GitHubServer,
    project_id: &str,
    field_id: &str,
    iteration_title: &str,
) -> Result<String, ErrorData> {
    let fields = get_project_fields(server, project_id).await?;
    let iteration_title_lower = iteration_title.to_lowercase();

    let field = fields
        .iter()
        .find(|field| field.get("id").and_then(Value::as_str) == Some(field_id))
        .ok_or_else(|| invalid_request("Project field was not found"))?;

    let empty = Vec::new();
    let iterations = field
        .pointer("/configuration/iterations")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let completed_iterations = field
        .pointer("/configuration/completedIterations")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    iterations
        .iter()
        .chain(completed_iterations.iter())
        .find(|iteration| {
            iteration
                .get("title")
                .and_then(Value::as_str)
                .map(|title| title.to_lowercase() == iteration_title_lower)
                .unwrap_or(false)
        })
        .and_then(|iteration| iteration.get("id").and_then(Value::as_str))
        .map(ToString::to_string)
        .ok_or_else(|| invalid_request(&format!("Iteration '{iteration_title}' was not found")))
}

async fn update_project_field(
    server: &GitHubServer,
    project_id: &str,
    item_id: &str,
    field_id: &str,
    value: Value,
) -> Result<Value, ErrorData> {
    let query = r#"
mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $value: ProjectV2FieldValue!) {
  updateProjectV2ItemFieldValue(
    input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: $value
    }
  ) {
    projectV2Item {
      id
      fieldValues(first: 100) {
        nodes {
          ... on ProjectV2ItemFieldTextValue {
            text
            field {
              ... on ProjectV2FieldCommon {
                id
                name
                dataType
              }
            }
          }
          ... on ProjectV2ItemFieldNumberValue {
            number
            field {
              ... on ProjectV2FieldCommon {
                id
                name
                dataType
              }
            }
          }
          ... on ProjectV2ItemFieldDateValue {
            date
            field {
              ... on ProjectV2FieldCommon {
                id
                name
                dataType
              }
            }
          }
          ... on ProjectV2ItemFieldSingleSelectValue {
            name
            optionId
            field {
              ... on ProjectV2FieldCommon {
                id
                name
                dataType
              }
            }
          }
          ... on ProjectV2ItemFieldIterationValue {
            title
            iterationId
            startDate
            duration
            field {
              ... on ProjectV2FieldCommon {
                id
                name
                dataType
              }
            }
          }
        }
      }
    }
  }
}
"#;

    let result = server
        .client
        .graphql(
            query,
            json!({
                "projectId": project_id,
                "itemId": item_id,
                "fieldId": field_id,
                "value": value,
            }),
        )
        .await?;

    Ok(result)
}

fn require_param(value: Option<String>, message: &str) -> Result<String, ErrorData> {
    value.ok_or_else(|| invalid_request(message))
}

fn invalid_request(message: &str) -> ErrorData {
    ErrorData::internal_error(message.to_string(), None)
}
