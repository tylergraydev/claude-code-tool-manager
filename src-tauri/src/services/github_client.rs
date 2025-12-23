use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GITHUB_API_BASE: &str = "https://api.github.com";
const RAW_GITHUB_BASE: &str = "https://raw.githubusercontent.com";
const USER_AGENT: &str = "claude-code-tool-manager/1.0";

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
    api_base: String,
    raw_base: String,
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

        Self {
            client,
            token,
            api_base: GITHUB_API_BASE.to_string(),
            raw_base: RAW_GITHUB_BASE.to_string(),
        }
    }

    /// Create a client with custom base URLs (for testing)
    #[cfg(test)]
    pub fn with_base_urls(token: Option<String>, api_base: String, raw_base: String) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            token,
            api_base,
            raw_base,
        }
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
        let url = format!("{}/repos/{}/{}", self.api_base, owner, repo);

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
    pub async fn get_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<Vec<ContentEntry>> {
        let url = if path.is_empty() {
            format!("{}/repos/{}/{}/contents", self.api_base, owner, repo)
        } else {
            format!(
                "{}/repos/{}/{}/contents/{}",
                self.api_base, owner, repo, path
            )
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
            let url = format!("{}/{}/{}/{}/README.md", self.raw_base, owner, repo, branch);

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
            let url = format!("{}/{}/{}/{}/{}", self.raw_base, owner, repo, branch, path);

            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                return Ok(response.text().await?);
            }
        }

        Err(anyhow!(
            "Could not find file {} in main or master branch",
            path
        ))
    }

    /// Get rate limit information
    pub async fn get_rate_limit(&self) -> Result<(i32, i32, i64)> {
        let url = format!("{}/rate_limit", self.api_base);

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
    pub async fn get_markdown_files_in_dir(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<Vec<(String, String)>> {
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
    pub async fn get_markdown_files(
        &self,
        owner: &str,
        repo: &str,
        dirs: &[&str],
    ) -> Result<Vec<(String, String)>> {
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
    use rstest::rstest;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // =========================================================================
    // parse_github_url tests
    // =========================================================================

    #[rstest]
    #[case("https://github.com/owner/repo", Some(("owner", "repo")))]
    #[case("http://github.com/owner/repo", Some(("owner", "repo")))]
    #[case("https://github.com/owner/repo.git", Some(("owner", "repo")))]
    #[case("github.com/owner/repo", Some(("owner", "repo")))]
    #[case("https://github.com/wong2/awesome-mcp-servers", Some(("wong2", "awesome-mcp-servers")))]
    #[case("https://github.com/wshobson/commands.git", Some(("wshobson", "commands")))]
    fn test_parse_github_url_valid(#[case] url: &str, #[case] expected: Option<(&str, &str)>) {
        let expected = expected.map(|(o, r)| (o.to_string(), r.to_string()));
        assert_eq!(parse_github_url(url), expected);
    }

    #[rstest]
    #[case("https://github.com/owner/repo/tree/main", Some(("owner", "repo")))]
    #[case("https://github.com/owner/repo/blob/main/README.md", Some(("owner", "repo")))]
    #[case("https://github.com/owner/repo/issues", Some(("owner", "repo")))]
    #[case("https://github.com/owner/repo/pulls", Some(("owner", "repo")))]
    fn test_parse_github_url_with_paths(#[case] url: &str, #[case] expected: Option<(&str, &str)>) {
        let expected = expected.map(|(o, r)| (o.to_string(), r.to_string()));
        assert_eq!(parse_github_url(url), expected);
    }

    #[rstest]
    #[case("https://gitlab.com/owner/repo", None)]
    #[case("https://bitbucket.org/owner/repo", None)]
    #[case("invalid-url", None)]
    #[case("https://github.com/", None)]
    #[case("https://github.com/owner", None)]
    #[case("", None)]
    fn test_parse_github_url_invalid(#[case] url: &str, #[case] expected: Option<(&str, &str)>) {
        let expected: Option<(String, String)> =
            expected.map(|(o, r)| (o.to_string(), r.to_string()));
        assert_eq!(parse_github_url(url), expected);
    }

    #[test]
    fn test_parse_github_url_whitespace() {
        assert_eq!(
            parse_github_url("  https://github.com/owner/repo  "),
            Some(("owner".to_string(), "repo".to_string()))
        );
    }

    #[test]
    fn test_parse_github_url_trailing_git() {
        assert_eq!(
            parse_github_url("https://github.com/owner/repo.git"),
            Some(("owner".to_string(), "repo".to_string()))
        );
    }

    // =========================================================================
    // GitHubClient construction tests
    // =========================================================================

    #[test]
    fn test_github_client_new_without_token() {
        let client = GitHubClient::new(None);
        assert!(client.token.is_none());
    }

    #[test]
    fn test_github_client_new_with_token() {
        let client = GitHubClient::new(Some("test-token".to_string()));
        assert_eq!(client.token, Some("test-token".to_string()));
    }

    // =========================================================================
    // ContentEntry deserialization tests
    // =========================================================================

    #[test]
    fn test_content_entry_deserialization() {
        let json = r#"{
            "name": "README.md",
            "path": "README.md",
            "type": "file",
            "size": 1234,
            "sha": "abc123",
            "url": "https://api.github.com/repos/owner/repo/contents/README.md",
            "html_url": "https://github.com/owner/repo/blob/main/README.md",
            "download_url": "https://raw.githubusercontent.com/owner/repo/main/README.md"
        }"#;

        let entry: ContentEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "README.md");
        assert_eq!(entry.entry_type, "file");
        assert_eq!(entry.size, Some(1234));
    }

    #[test]
    fn test_content_entry_directory() {
        let json = r#"{
            "name": "src",
            "path": "src",
            "type": "dir",
            "sha": "def456",
            "url": "https://api.github.com/repos/owner/repo/contents/src"
        }"#;

        let entry: ContentEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.entry_type, "dir");
        assert!(entry.size.is_none());
        assert!(entry.download_url.is_none());
    }

    // =========================================================================
    // RateLimitResponse deserialization tests
    // =========================================================================

    #[test]
    fn test_rate_limit_response_deserialization() {
        let json = r#"{
            "resources": {
                "core": {
                    "limit": 60,
                    "remaining": 55,
                    "reset": 1700000000
                }
            }
        }"#;

        let response: RateLimitResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.resources.core.limit, 60);
        assert_eq!(response.resources.core.remaining, 55);
        assert_eq!(response.resources.core.reset, 1700000000);
    }

    // =========================================================================
    // HTTP client tests with wiremock
    // =========================================================================

    #[tokio::test]
    async fn test_get_repo_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/test-repo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "test-repo",
                "full_name": "owner/test-repo",
                "description": "A test repository",
                "html_url": "https://github.com/owner/test-repo",
                "stargazers_count": 42,
                "default_branch": "main"
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let repo = client.get_repo("owner", "test-repo").await.unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.full_name, "owner/test-repo");
        assert_eq!(repo.description, Some("A test repository".to_string()));
        assert_eq!(repo.stargazers_count, 42);
        assert_eq!(repo.default_branch, "main");
    }

    #[tokio::test]
    async fn test_get_repo_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let result = client.get_repo("owner", "nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_get_contents_files() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents/src"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "main.rs",
                    "path": "src/main.rs",
                    "type": "file",
                    "size": 1234,
                    "sha": "abc123",
                    "url": "https://api.github.com/repos/owner/repo/contents/src/main.rs"
                },
                {
                    "name": "lib.rs",
                    "path": "src/lib.rs",
                    "type": "file",
                    "size": 567,
                    "sha": "def456",
                    "url": "https://api.github.com/repos/owner/repo/contents/src/lib.rs"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let contents = client.get_contents("owner", "repo", "src").await.unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0].name, "main.rs");
        assert_eq!(contents[0].entry_type, "file");
        assert_eq!(contents[1].name, "lib.rs");
    }

    #[tokio::test]
    async fn test_get_contents_root() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "README.md",
                    "path": "README.md",
                    "type": "file",
                    "sha": "abc123",
                    "url": "https://api.github.com/repos/owner/repo/contents/README.md"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let contents = client.get_contents("owner", "repo", "").await.unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].name, "README.md");
    }

    #[tokio::test]
    async fn test_get_readme_main_branch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/README.md"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("# Test Repo\n\nThis is a test."),
            )
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let readme = client.get_readme("owner", "repo").await.unwrap();
        assert!(readme.contains("# Test Repo"));
        assert!(readme.contains("This is a test."));
    }

    #[tokio::test]
    async fn test_get_readme_fallback_to_master() {
        let mock_server = MockServer::start().await;

        // main branch returns 404
        Mock::given(method("GET"))
            .and(path("/owner/repo/main/README.md"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        // master branch succeeds
        Mock::given(method("GET"))
            .and(path("/owner/repo/master/README.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Master Branch"))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let readme = client.get_readme("owner", "repo").await.unwrap();
        assert!(readme.contains("# Master Branch"));
    }

    #[tokio::test]
    async fn test_get_readme_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/README.md"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/master/README.md"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let result = client.get_readme("owner", "repo").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find README.md"));
    }

    #[tokio::test]
    async fn test_get_file_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/src/config.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"key": "value"}"#))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let content = client
            .get_file("owner", "repo", "src/config.json")
            .await
            .unwrap();
        assert!(content.contains(r#""key": "value""#));
    }

    #[tokio::test]
    async fn test_get_file_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/nonexistent.txt"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/master/nonexistent.txt"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let result = client.get_file("owner", "repo", "nonexistent.txt").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find file"));
    }

    #[tokio::test]
    async fn test_get_rate_limit_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rate_limit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resources": {
                    "core": {
                        "limit": 5000,
                        "remaining": 4999,
                        "reset": 1700000000
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let (limit, remaining, reset) = client.get_rate_limit().await.unwrap();
        assert_eq!(limit, 5000);
        assert_eq!(remaining, 4999);
        assert_eq!(reset, 1700000000);
    }

    #[tokio::test]
    async fn test_get_rate_limit_unauthenticated() {
        let mock_server = MockServer::start().await;

        // Unauthenticated users get 200 OK but with lower limits in response body
        Mock::given(method("GET"))
            .and(path("/rate_limit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resources": {
                    "core": {
                        "limit": 60,
                        "remaining": 58,
                        "reset": 1700000000
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        // Unauthenticated users have lower rate limit (60 vs 5000)
        let (limit, remaining, _) = client.get_rate_limit().await.unwrap();
        assert_eq!(limit, 60);
        assert_eq!(remaining, 58);
    }

    #[tokio::test]
    async fn test_get_markdown_files_in_dir() {
        let mock_server = MockServer::start().await;

        // Mock the contents API
        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents/docs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "guide.md",
                    "path": "docs/guide.md",
                    "type": "file",
                    "sha": "abc123",
                    "url": "https://api.github.com/repos/owner/repo/contents/docs/guide.md"
                },
                {
                    "name": "api.md",
                    "path": "docs/api.md",
                    "type": "file",
                    "sha": "def456",
                    "url": "https://api.github.com/repos/owner/repo/contents/docs/api.md"
                },
                {
                    "name": "image.png",
                    "path": "docs/image.png",
                    "type": "file",
                    "sha": "ghi789",
                    "url": "https://api.github.com/repos/owner/repo/contents/docs/image.png"
                }
            ])))
            .mount(&mock_server)
            .await;

        // Mock the raw file fetches
        Mock::given(method("GET"))
            .and(path("/owner/repo/main/docs/guide.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Guide\n\nWelcome!"))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/docs/api.md"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("# API Reference\n\nEndpoints..."),
            )
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let files = client
            .get_markdown_files_in_dir("owner", "repo", "docs")
            .await
            .unwrap();

        // Should only get .md files, not .png
        assert_eq!(files.len(), 2);

        // Find guide.md
        let guide = files.iter().find(|(path, _)| path == "docs/guide.md");
        assert!(guide.is_some());
        assert!(guide.unwrap().1.contains("# Guide"));

        // Find api.md
        let api = files.iter().find(|(path, _)| path == "docs/api.md");
        assert!(api.is_some());
        assert!(api.unwrap().1.contains("# API Reference"));
    }

    #[tokio::test]
    async fn test_get_markdown_files_multiple_dirs() {
        let mock_server = MockServer::start().await;

        // Mock docs directory
        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents/docs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "intro.md",
                    "path": "docs/intro.md",
                    "type": "file",
                    "sha": "abc123",
                    "url": "https://api.github.com/repos/owner/repo/contents/docs/intro.md"
                }
            ])))
            .mount(&mock_server)
            .await;

        // Mock guides directory
        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents/guides"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "setup.md",
                    "path": "guides/setup.md",
                    "type": "file",
                    "sha": "def456",
                    "url": "https://api.github.com/repos/owner/repo/contents/guides/setup.md"
                }
            ])))
            .mount(&mock_server)
            .await;

        // Mock raw file fetches
        Mock::given(method("GET"))
            .and(path("/owner/repo/main/docs/intro.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Intro"))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/owner/repo/main/guides/setup.md"))
            .respond_with(ResponseTemplate::new(200).set_body_string("# Setup Guide"))
            .mount(&mock_server)
            .await;

        let client = GitHubClient::with_base_urls(None, mock_server.uri(), mock_server.uri());

        let files = client
            .get_markdown_files("owner", "repo", &["docs", "guides"])
            .await
            .unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_with_base_urls_constructor() {
        let client = GitHubClient::with_base_urls(
            Some("test-token".to_string()),
            "http://localhost:8080".to_string(),
            "http://localhost:8081".to_string(),
        );

        assert_eq!(client.token, Some("test-token".to_string()));
        assert_eq!(client.api_base, "http://localhost:8080");
        assert_eq!(client.raw_base, "http://localhost:8081");
    }

    #[test]
    fn test_default_base_urls() {
        let client = GitHubClient::new(None);
        assert_eq!(client.api_base, "https://api.github.com");
        assert_eq!(client.raw_base, "https://raw.githubusercontent.com");
    }
}
