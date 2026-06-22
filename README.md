# GitHub MCP
A local Rust-based model context protocol (MCP) server that exposes GitHub repository, issue, and pull request operations over stdio.

## Problem Solved
`github-mcp` lets MCP-compatible clients interact with GitHub through structured tools. It usses the GitHub REST API and can run with or
without a `GITHUB_TOKEN`, though authenitcated requests have higher rate limits and access to private resources when permitted.

## Features

### Implemented
- Search GitHub repositories
- Fetch repository details
- List repository issues
- Fetch a specific issue
- Create a new issue
- Create a new PR
- List repository pull requests (PRs)
- Fetch a specific PR
- Return structured JSON responses from the GitHub API
- Support optional GitHub token authentication

### Planned
- Fetch and create issue and PR comments
- Fetch a list of remote branches for a repository
- Fetch a list of commits and their details that have been pushed to the repository
- Fetch a list of releases and the release details

## Requirements
- Rust toolchain compatible with Rust 2024 edition
- Network access to `https://api.github.com/`
- Optional: a GitHub personal access token provided through `GITHUB_TOKEN`
  - If a PAT is not provided the following will not be available:
    - Access to your private repos
    - Write priviledged tools (create a new issue, create a PR, create comments on issues/PR)
  - A `GITHUB_TOKEN` is optional, but recommended, because unauthenticated GitHub API requests have lower rate limits.

## Build/Installation
Clone the repository and build the binary:

```bash
# Clone the repository
git clone https://github.com/bhyland-usda/github-mcp.git
cd ./github-mcp

# Build the binary
cargo build --release
```

The binary will be located in github-mcp/target/release.

---

```bash
git clone https://github.com/bhyland-usda/github-mcp.git
cd ./github-mcp

# Install to your local .cargo/bin directory, which should be on your PATH already
cargo install --path . 
```

## Configuration
`github-mcp` reads configuration from environment variables.

| Variable | Required | Description |
| --- | --- | --- |
| GITHUB_TOKEN | No | GitHub personal access token for authenticated API requests. Recommended for higher rate limits, private repos, and write operations. |

## Usage
Since MCP clients differ, the following is a generic config-style example to get into your specific client's settings.json file, whatever that may be named:

```json
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
```

If you'd rather not copy/paste your PAT in a plain-text file (recommended not to), it's preferred to load `GITHUB_TOKEN` from your shell,
secret manager, or MCP client environment configuration rather than commit it to the settings.json file.

## Available Tools

| Tool | Description | Key Parameters |
| --- | --- | --- |
| search_repositories | Search GitHub repositories using GitHub's search API. | `query`, `per_page`, `page` |
| get_repository | Get details for a repository. | `owner`, `repo` |
| list_issues | List issues for a repository. | `owner`, `repo`, `state`, `per_page`, `page` |
| get_issue | Get a specific issue. | `owner`, `repo`, `issue_number` |
| create_issue | Create a new issue. | `owner`, `repo`, `title`, `body`, `labels`, `assignees` |
| create_pull_request | Create a new pull requst for a repository. | `owner`, `repo`, `title`, `body`, `issue` |
| list_pull_requests | List pull requests for a repository. | `owner`, `repo`, `state`, `per_page`, `page` |
| get_pull_request | Get a specific pull request. | `owner`, `repo`, `pull_number` |

For list operations, `per_page` defaults to 30 and is clamped between 1 and 100. `page` defaults to 1. State filters accept `open`, `closed`,
or `all`; invalid or missing values default to `open`.

## Development
Command development commands:

```bash
cargo fmt
cargo check
cargo test
cargo build
cargo build --release
cargo run
cargo run --release
cargo clippy
```

**NOTE**: *`cargo test` can be used as tests are added.*

## Errors and Limitations
- Pagination is controlled manually with `page` and `per_page`.
- The server does NOT cache GitHub API responses.
- GitHub API responses are returned to the MCP client as tool errors with details.
- Only repository search/details, issues, and pull requests are currently supported.
