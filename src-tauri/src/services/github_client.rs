use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GITHUB_API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "claude-code-tool-manager/1.0";

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub stargazers_count: i32,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String, // "file" or "dir"
    pub size: Option<i64>,
    pub sha: String,
    pub url: String,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: i64,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub resources: RateLimitResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResources {
    pub core: RateLimitCore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitCore {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, token }
    }

    fn build_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        if let Some(ref token) = self.token {
            if let Ok(auth_value) = header::HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(header::AUTHORIZATION, auth_value);
            }
        }

        headers
    }

    /// Get repository information
    #[allow(dead_code)]
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<RepoInfo> {
        let url = format!("{}/repos/{}/{}", GITHUB_API_BASE, owner, repo);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }

        let repo_info: RepoInfo = response.json().await?;
        Ok(repo_info)
    }

    /// Get repository contents (files and directories)
    pub async fn get_contents(&self, owner: &str, repo: &str, path: &str) -> Result<Vec<ContentEntry>> {
        let url = if path.is_empty() {
            format!("{}/repos/{}/{}/contents", GITHUB_API_BASE, owner, repo)
        } else {
            format!("{}/repos/{}/{}/contents/{}", GITHUB_API_BASE, owner, repo, path)
        };

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }

        let contents: Vec<ContentEntry> = response.json().await?;
        Ok(contents)
    }

    /// Get README content using raw.githubusercontent.com (no rate limit)
    pub async fn get_readme(&self, owner: &str, repo: &str) -> Result<String> {
        // Try main branch first, then master
        for branch in ["main", "master"] {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/README.md",
                owner, repo, branch
            );

            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                return Ok(response.text().await?);
            }
        }

        Err(anyhow!("Could not find README.md in main or master branch"))
    }

    /// Get file content by path using raw.githubusercontent.com (no rate limit)
    pub async fn get_file(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        // Try main branch first, then master
        for branch in ["main", "master"] {
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repo, branch, path
            );

            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                return Ok(response.text().await?);
            }
        }

        Err(anyhow!("Could not find file {} in main or master branch", path))
    }

    /// Get rate limit information
    pub async fn get_rate_limit(&self) -> Result<(i32, i32, i64)> {
        let url = format!("{}/rate_limit", GITHUB_API_BASE);

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }

        let rate_limit: RateLimitResponse = response.json().await?;
        Ok((
            rate_limit.resources.core.limit,
            rate_limit.resources.core.remaining,
            rate_limit.resources.core.reset,
        ))
    }

    /// Decode base64 content from GitHub API response
    #[allow(dead_code)]
    fn decode_content(&self, file_content: &FileContent) -> Result<String> {
        match (&file_content.content, &file_content.encoding) {
            (Some(content), Some(encoding)) if encoding == "base64" => {
                // Remove newlines from base64 content
                let clean_content = content.replace('\n', "").replace('\r', "");
                let decoded = STANDARD.decode(&clean_content)?;
                Ok(String::from_utf8(decoded)?)
            }
            (Some(content), _) => Ok(content.clone()),
            (None, _) => Err(anyhow!("No content in file response")),
        }
    }

    /// Get all markdown files in a directory (non-recursive, single level)
    pub async fn get_markdown_files_in_dir(&self, owner: &str, repo: &str, path: &str) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();
        let contents = self.get_contents(owner, repo, path).await?;

        for entry in contents {
            if entry.entry_type == "file" && entry.name.ends_with(".md") {
                if let Ok(content) = self.get_file(owner, repo, &entry.path).await {
                    files.push((entry.path, content));
                }
            }
        }

        Ok(files)
    }

    /// Get markdown files from multiple directories
    pub async fn get_markdown_files(&self, owner: &str, repo: &str, dirs: &[&str]) -> Result<Vec<(String, String)>> {
        let mut all_files = Vec::new();

        for dir in dirs {
            if let Ok(files) = self.get_markdown_files_in_dir(owner, repo, dir).await {
                all_files.extend(files);
            }
        }

        Ok(all_files)
    }
}

/// Parse GitHub URL to extract owner and repo
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Handle URLs like:
    // https://github.com/owner/repo
    // https://github.com/owner/repo.git
    // github.com/owner/repo

    let url = url.trim().trim_end_matches(".git");

    let parts: Vec<&str> = url.split('/').collect();

    // Find github.com in the path
    if let Some(github_pos) = parts.iter().position(|&p| p == "github.com") {
        if parts.len() > github_pos + 2 {
            let owner = parts[github_pos + 1].to_string();
            let repo = parts[github_pos + 2].to_string();
            if !owner.is_empty() && !repo.is_empty() {
                return Some((owner, repo));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        assert_eq!(
            parse_github_url("https://github.com/wong2/awesome-mcp-servers"),
            Some(("wong2".to_string(), "awesome-mcp-servers".to_string()))
        );

        assert_eq!(
            parse_github_url("https://github.com/wshobson/commands.git"),
            Some(("wshobson".to_string(), "commands".to_string()))
        );

        assert_eq!(
            parse_github_url("github.com/owner/repo"),
            Some(("owner".to_string(), "repo".to_string()))
        );
    }
}
