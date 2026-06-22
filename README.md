# GitHub MCP

A local Rust-based Model Context Protocol (MCP) server that exposes GitHub repository, issue, pull request, and label operations over stdio.

## Problem Solved

`github-mcp` lets MCP-compatible clients interact with GitHub through structured tools backed by the GitHub REST API. It can run with or without a `GITHUB_TOKEN`, though authenticated requests have higher rate limits and can access private resources when the token has the required permissions.

## Features

- Runs as an MCP server over stdio.
- Uses the GitHub REST API with structured JSON responses.
- Supports optional GitHub token authentication through `GITHUB_TOKEN`.
- Supports unauthenticated read-only access for public GitHub resources, subject to GitHub rate limits.
- Provides repository search and repository detail tools.
- Provides issue listing, issue retrieval, and issue creation tools.
- Provides pull request listing, pull request retrieval, and pull request creation tools.
- Provides repository label management tools.
- Provides issue label management tools.
- Supports manual pagination with `page` and `per_page`.
- Normalizes issue and pull request state filters to `open`, `closed`, or `all`.
- Reports GitHub API failures back to the MCP client with status and response details.
- Uses request timeouts to avoid hanging indefinitely on GitHub API calls.

## Requirements

- Rust toolchain compatible with Rust 2024 edition.
- Network access to `https://api.github.com/`.
- Optional: a GitHub personal access token provided through `GITHUB_TOKEN`.

A `GITHUB_TOKEN` is optional, but recommended. Without a token:

- GitHub API rate limits are lower.
- Private repositories are not available.
- Write operations such as creating issues, creating pull requests, and modifying labels may fail.

## Build and Installation

Clone the repository and build the release binary:

    git clone https://github.com/bhyland-usda/github-mcp.git
    cd ./github-mcp
    cargo build --release

The binary will be available under:

    target/release/github-mcp

You can also install it into your local Cargo binary directory:

    git clone https://github.com/bhyland-usda/github-mcp.git
    cd ./github-mcp
    cargo install --path .

## Configuration

`github-mcp` reads configuration from environment variables.

| Variable | Required | Description |
| --- | --- | --- |
| `GITHUB_TOKEN` | No | GitHub personal access token for authenticated API requests. Recommended for higher rate limits, private repositories, and write operations. |
| `RUST_LOG` | No | Optional logging configuration used by `tracing_subscriber`. Defaults to info-level logging. |

## Usage

MCP client configuration varies by client. A generic configuration example looks like this:

    {
      "mcpServers": {
        "github": {
          "command": "/path/to/github-mcp",
          "env": {
            "GITHUB_TOKEN": "your-github-token"
          }
        }
      }
    }

For better security, avoid storing a personal access token in a plain-text settings file when possible. Prefer loading `GITHUB_TOKEN` from your shell environment, secret manager, or MCP client environment configuration.

## Available Tools

### Repository Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| `search_repositories` | Search GitHub repositories using GitHub's search API. | `query`, `per_page`, `page` |
| `get_repository` | Get details for a specific repository. | `owner`, `repo` |

### Repository Label Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| `get_repository_labels` | List labels for a repository. | `owner`, `repo` |
| `create_repository_labels` | Create a label in a repository. | `owner`, `repo`, `name`, `color`, `description` |
| `update_repository_label` | Update a repository label's name, color, and/or description. | `owner`, `repo`, `name`, `new_name`, `color`, `description` |
| `delete_repository_label` | Delete a specific label from a repository. | `owner`, `repo`, `name` |

### Issue Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| `list_issues` | List issues for a repository. | `owner`, `repo`, `state`, `per_page`, `page` |
| `get_issue` | Get a specific issue. | `owner`, `repo`, `issue_number` |
| `create_issue` | Create a new issue. | `owner`, `repo`, `title`, `body`, `labels`, `assignees` |

### Issue Label Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| `list_issue_lables` | List labels attached to a specific issue. | `owner`, `repo`, `issue_number` |
| `replace_issue_labels` | Replace labels on a specific issue. | `owner`, `repo`, `issue_number`, `labels` |
| `delete_issue_labels` | Delete all labels from a specific issue. | `owner`, `repo`, `issue_number` |
| `add_issue_labels` | Add one or more labels to a specific issue. | `owner`, `repo`, `issue_number`, `labels` |

Note: `list_issue_lables` is exposed with the current tool name spelling used by the server.

### Pull Request Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| `list_pull_requests` | List pull requests for a repository. | `owner`, `repo`, `state`, `per_page`, `page` |
| `get_pull_request` | Get a specific pull request. | `owner`, `repo`, `pull_number` |
| `create_pull_request` | Create a new pull request for a repository. | `owner`, `repo`, `title`, `body`, `head`, `base`, `issue` |

## Parameter Notes

For list operations:

- `per_page` defaults to `30`.
- `per_page` is clamped between `1` and `100`.
- `page` defaults to `1`.

For issue and pull request state filters:

- Supported values are `open`, `closed`, and `all`.
- Invalid or missing values default to `open`.

For pull request creation:

- `head` is the source branch. For forks, use GitHub's `owner:branch` format.
- `base` is the target branch.
- `issue` is optional and can be used when creating a pull request associated with an existing issue.

For label colors:

- GitHub expects hexadecimal colors without the leading `#`.

## Authentication and Permissions

Read-only tools can often work against public repositories without authentication. Authenticated requests are recommended for all use cases and are required for many write operations.

The token used with `GITHUB_TOKEN` should have permissions appropriate for the operations you plan to perform. For example:

- Reading private repositories requires repository read access.
- Creating issues requires issue write access.
- Creating pull requests requires pull request write access.
- Creating, updating, or deleting labels requires repository metadata/administration permissions as enforced by GitHub.

Do not hardcode personal access tokens into source files or commit them to version control.

## Development

Common development commands:

    cargo fmt
    cargo check
    cargo test
    cargo build
    cargo build --release
    cargo run
    cargo run --release
    cargo clippy

## Errors and Limitations

- Pagination is controlled manually with `page` and `per_page`.
- The server does not cache GitHub API responses.
- GitHub API errors are returned to the MCP client with HTTP status and response details.
- The server currently focuses on repositories, issues, pull requests, repository labels, and issue labels.
- Branch, commit, release, and comment tools are not currently implemented.

## Planned Enhancements

- Fetch and create issue comments.
- Fetch and create pull request comments.
- Fetch remote branches for a repository.
- Fetch commits and commit details.
- Fetch releases and release details.
