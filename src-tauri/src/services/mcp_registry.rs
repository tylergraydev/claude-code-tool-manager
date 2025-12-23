use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const REGISTRY_BASE_URL: &str = "https://registry.modelcontextprotocol.io";
const USER_AGENT: &str = "claude-code-tool-manager/1.2";

// ============================================================================
// Registry API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RegistryResponse {
    pub servers: Vec<ServerWrapper>,
    pub metadata: Option<RegistryMetadata>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ServerWrapper {
    pub server: RegistryServer,
    #[serde(rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RegistryMetadata {
    pub count: Option<u32>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryServer {
    pub name: String,
    pub description: Option<String>,
    pub repository: Option<RepositoryInfo>,
    pub version: Option<String>,
    pub packages: Option<Vec<Package>>,
    pub remotes: Option<Vec<Remote>>,
    // Populated from _meta after parsing
    #[serde(skip)]
    pub updated_at: Option<String>,
    // Catch-all for unknown fields (like $schema, icons, title, etc.)
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryInfo {
    pub url: Option<String>,
    #[allow(dead_code)]
    pub source: Option<String>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub registry_type: String,      // "npm", "pypi", "nuget", "oci", "mcpb"
    pub identifier: Option<String>, // Package identifier (e.g., "npm:@org/package")
    pub name: Option<String>,
    #[allow(dead_code)]
    pub version: Option<String>,
    pub arguments: Option<Vec<PackageArgument>>,
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
    #[allow(dead_code)]
    pub transport: Option<Transport>,
    // Catch-all for unknown fields
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Transport {
    #[serde(rename = "type")]
    pub transport_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageArgument {
    #[serde(rename = "type")]
    pub arg_type: Option<String>, // "positional" or "named"
    pub name: Option<String>,
    pub value: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub is_required: Option<bool>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentVariable {
    pub name: String,
    pub description: Option<String>,
    pub is_required: Option<bool>,
    #[allow(dead_code)]
    pub is_secret: Option<bool>,
    pub default: Option<String>,
    #[allow(dead_code)]
    pub format: Option<String>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Remote {
    #[serde(rename = "type")]
    pub transport_type: String, // "sse", "http", "streamable-http"
    pub url: String,
    pub headers: Option<Vec<RemoteHeader>>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHeader {
    pub name: String,
    pub value: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub is_required: Option<bool>,
    #[allow(dead_code)]
    pub is_secret: Option<bool>,
}

// ============================================================================
// Converted MCP Entry (matches your app's MCP schema)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryMcpEntry {
    pub registry_id: String,
    pub name: String,
    pub description: Option<String>,
    pub mcp_type: String, // "stdio", "sse", "http"
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub env_placeholders: Option<Vec<EnvPlaceholder>>,
    pub source_url: Option<String>,
    pub version: Option<String>,
    pub registry_type: Option<String>, // "npm", "pypi", etc.
    pub updated_at: Option<String>,    // ISO timestamp from registry
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvPlaceholder {
    pub name: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub default: Option<String>,
}

// ============================================================================
// Registry Client
// ============================================================================

pub struct RegistryClient {
    client: Client,
    base_url: String,
}

impl RegistryClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: REGISTRY_BASE_URL.to_string(),
        }
    }

    /// Create a client with a custom base URL (for testing)
    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
        }
    }

    /// Search for MCP servers in the registry
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<RegistryServer>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/v0/servers?search={}&limit={}&status=active&version=latest",
            self.base_url, encoded_query, limit
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Registry API error {}: {}", status, text));
        }

        // Parse as dynamic JSON for resilience
        let json: serde_json::Value = response.json().await?;
        let servers_array = json
            .get("servers")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("No servers array in response"))?;

        let mut servers = Vec::new();
        for item in servers_array {
            if let Some(server_obj) = item.get("server") {
                if let Ok(server) = serde_json::from_value::<RegistryServer>(server_obj.clone()) {
                    servers.push(server);
                }
            }
        }

        Ok(servers)
    }

    /// List servers with pagination
    pub async fn list(
        &self,
        limit: u32,
        cursor: Option<&str>,
    ) -> Result<(Vec<RegistryServer>, Option<String>)> {
        let mut url = format!(
            "{}/v0/servers?limit={}&status=active&version=latest",
            self.base_url, limit
        );

        if let Some(c) = cursor {
            url.push_str(&format!("&cursor={}", urlencoding::encode(c)));
        }

        log::info!("[Registry] Fetching: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Registry API error {}: {}", status, text));
        }

        // Parse as dynamic JSON to be more resilient
        let json: serde_json::Value = response.json().await?;

        // Extract next cursor from metadata
        let next_cursor = json
            .get("metadata")
            .and_then(|m| m.get("nextCursor"))
            .and_then(|c| c.as_str())
            .map(String::from);

        // Extract servers array
        let servers_array = json
            .get("servers")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("No servers array in response"))?;

        eprintln!(
            "[Registry] API returned {} servers in response",
            servers_array.len()
        );

        // Parse each server dynamically (API already filters for latest versions via version=latest param)
        let mut servers = Vec::new();
        let mut parse_failures = 0;
        for item in servers_array {
            // Each item has { server: {...}, _meta: {...} }
            if let Some(server_obj) = item.get("server") {
                match serde_json::from_value::<RegistryServer>(server_obj.clone()) {
                    Ok(mut server) => {
                        // Extract updatedAt from _meta
                        server.updated_at = item
                            .get("_meta")
                            .and_then(|m| m.get("io.modelcontextprotocol.registry/official"))
                            .and_then(|o| o.get("updatedAt"))
                            .and_then(|u| u.as_str())
                            .map(String::from);
                        servers.push(server);
                    }
                    Err(e) => {
                        parse_failures += 1;
                        let name = server_obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");
                        // Log first few failures in detail
                        if parse_failures <= 3 {
                            eprintln!("[Registry] PARSE FAILED '{}': {}", name, e);
                            eprintln!(
                                "[Registry] Server JSON: {}",
                                serde_json::to_string_pretty(server_obj).unwrap_or_default()
                            );
                        }
                    }
                }
            }
        }
        if parse_failures > 0 {
            eprintln!("[Registry] Total parse failures: {}", parse_failures);
        }

        eprintln!("[Registry] Successfully parsed {} servers", servers.len());

        Ok((servers, next_cursor))
    }

    /// Get a specific server by ID
    pub async fn get_server(&self, server_id: &str) -> Result<RegistryServer> {
        let url = format!("{}/v0/servers/{}", self.base_url, server_id);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            return Err(anyhow!("Server not found: {}", server_id));
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Registry API error {}: {}", status, text));
        }

        // Single server response is also wrapped
        let wrapper: ServerWrapper = response.json().await?;
        Ok(wrapper.server)
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Conversion Logic
// ============================================================================

impl RegistryServer {
    /// Convert a registry server to an MCP entry for the app
    pub fn to_mcp_entry(&self) -> Result<RegistryMcpEntry> {
        // Try packages first (local/stdio MCPs)
        if let Some(packages) = &self.packages {
            if let Some(package) = packages.first() {
                // If package conversion succeeds, return it
                // If it fails (unsupported registry type), fall through to try remotes
                if let Ok(entry) = package_to_mcp_entry(self, package) {
                    return Ok(entry);
                }
            }
        }

        // Try remotes (SSE/HTTP MCPs)
        if let Some(remotes) = &self.remotes {
            if let Some(remote) = remotes.first() {
                return Ok(remote_to_mcp_entry(self, remote));
            }
        }

        Err(anyhow!("Server has no packages or remotes defined"))
    }
}

fn extract_short_name(full_name: &str) -> String {
    // "io.github.user/my-mcp-server" -> "my-mcp-server"
    full_name.split('/').last().unwrap_or(full_name).to_string()
}

fn package_to_mcp_entry(server: &RegistryServer, package: &Package) -> Result<RegistryMcpEntry> {
    // Get package name from identifier or name field
    let pkg_name = package
        .identifier
        .as_ref()
        .or(package.name.as_ref())
        .ok_or_else(|| anyhow!("Missing package identifier or name"))?;

    // Extract just the package name from identifier like "docker.io/user/image:tag"
    let clean_pkg_name = extract_package_name(pkg_name, &package.registry_type);

    let (command, mut args) = match package.registry_type.as_str() {
        "npm" => ("npx".to_string(), vec!["-y".to_string(), clean_pkg_name]),
        "pypi" => ("uvx".to_string(), vec![clean_pkg_name]),
        "oci" | "docker" => (
            "docker".to_string(),
            vec![
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
                clean_pkg_name,
            ],
        ),
        other => {
            return Err(anyhow!("Unsupported registry type: {}", other));
        }
    };

    // Add package arguments (now using 'arguments' field)
    if let Some(pkg_args) = &package.arguments {
        for arg in pkg_args {
            match arg.arg_type.as_deref() {
                Some("positional") => {
                    if let Some(value) = &arg.value {
                        args.push(value.clone());
                    } else if let Some(default) = &arg.default {
                        args.push(default.clone());
                    }
                }
                Some("named") => {
                    if let Some(name) = &arg.name {
                        args.push(name.clone());
                        if let Some(value) = &arg.value {
                            args.push(value.clone());
                        } else if let Some(default) = &arg.default {
                            args.push(default.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Build environment variables map and placeholders
    let mut env = HashMap::new();
    let mut env_placeholders = Vec::new();

    if let Some(vars) = &package.environment_variables {
        for v in vars {
            let value = v.default.clone().unwrap_or_default();
            env.insert(v.name.clone(), value);

            env_placeholders.push(EnvPlaceholder {
                name: v.name.clone(),
                description: v.description.clone(),
                is_required: v.is_required.unwrap_or(false),
                default: v.default.clone(),
            });
        }
    }

    let env = if env.is_empty() { None } else { Some(env) };
    let env_placeholders = if env_placeholders.is_empty() {
        None
    } else {
        Some(env_placeholders)
    };

    Ok(RegistryMcpEntry {
        registry_id: server.name.clone(), // Use name as ID since there's no separate id field
        name: extract_short_name(&server.name),
        description: server.description.clone(),
        mcp_type: "stdio".to_string(),
        command: Some(command),
        args: Some(args),
        url: None,
        headers: None,
        env,
        env_placeholders,
        source_url: server.repository.as_ref().and_then(|r| r.url.clone()),
        version: server.version.clone(),
        registry_type: Some(package.registry_type.clone()),
        updated_at: server.updated_at.clone(),
    })
}

fn extract_package_name(identifier: &str, registry_type: &str) -> String {
    match registry_type {
        "oci" | "docker" => {
            // "docker.io/aliengiraffe/spotdb:0.1.0" -> "aliengiraffe/spotdb"
            // Remove registry prefix and version tag
            let without_registry = identifier
                .trim_start_matches("docker.io/")
                .trim_start_matches("ghcr.io/")
                .trim_start_matches("gcr.io/");
            // Remove version tag
            without_registry
                .split(':')
                .next()
                .unwrap_or(identifier)
                .to_string()
        }
        _ => identifier.to_string(),
    }
}

fn remote_to_mcp_entry(server: &RegistryServer, remote: &Remote) -> RegistryMcpEntry {
    let mcp_type = match remote.transport_type.as_str() {
        "sse" => "sse",
        "streamable-http" | "http" => "http",
        _ => "http",
    };

    // Convert Vec<RemoteHeader> to HashMap<String, String>
    let headers = remote.headers.as_ref().map(|hdrs| {
        hdrs.iter()
            .filter_map(|h| {
                // Use the value if present, otherwise use a placeholder
                let value = h
                    .value
                    .clone()
                    .unwrap_or_else(|| format!("${{{}}}", h.name));
                Some((h.name.clone(), value))
            })
            .collect::<HashMap<String, String>>()
    });

    RegistryMcpEntry {
        registry_id: server.name.clone(),
        name: extract_short_name(&server.name),
        description: server.description.clone(),
        mcp_type: mcp_type.to_string(),
        command: None,
        args: None,
        url: Some(remote.url.clone()),
        headers,
        env: None,
        env_placeholders: None,
        source_url: server.repository.as_ref().and_then(|r| r.url.clone()),
        version: server.version.clone(),
        registry_type: None,
        updated_at: server.updated_at.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_extract_short_name() {
        assert_eq!(
            extract_short_name("io.github.user/my-mcp-server"),
            "my-mcp-server"
        );
        assert_eq!(extract_short_name("simple-name"), "simple-name");
    }

    #[test]
    fn test_npm_package_conversion() {
        let server = RegistryServer {
            name: "io.github.test/filesystem-server".to_string(),
            description: Some("A filesystem MCP server".to_string()),
            repository: Some(RepositoryInfo {
                url: Some("https://github.com/test/filesystem".to_string()),
                source: None,
                extra: None,
            }),
            version: Some("1.0.0".to_string()),
            packages: Some(vec![Package {
                registry_type: "npm".to_string(),
                identifier: Some("@modelcontextprotocol/server-filesystem".to_string()),
                name: None,
                version: Some("1.0.0".to_string()),
                arguments: Some(vec![PackageArgument {
                    arg_type: Some("positional".to_string()),
                    name: None,
                    value: None,
                    description: Some("Directory to serve".to_string()),
                    is_required: Some(true),
                    default: Some("~/Documents".to_string()),
                }]),
                environment_variables: None,
                transport: None,
                extra: None,
            }]),
            remotes: None,
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "filesystem-server");
        assert_eq!(entry.mcp_type, "stdio");
        assert_eq!(entry.command, Some("npx".to_string()));

        let args = entry.args.unwrap();
        assert_eq!(args[0], "-y");
        assert_eq!(args[1], "@modelcontextprotocol/server-filesystem");
        assert_eq!(args[2], "~/Documents");
    }

    #[test]
    fn test_sse_remote_conversion() {
        let server = RegistryServer {
            name: "remote-server".to_string(),
            description: Some("A remote SSE server".to_string()),
            repository: None,
            version: None,
            packages: None,
            remotes: Some(vec![Remote {
                transport_type: "sse".to_string(),
                url: "https://mcp.example.com/sse".to_string(),
                headers: Some(vec![RemoteHeader {
                    name: "Authorization".to_string(),
                    value: Some("Bearer token".to_string()),
                    description: None,
                    is_required: None,
                    is_secret: None,
                }]),
                extra: None,
            }]),
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "remote-server");
        assert_eq!(entry.mcp_type, "sse");
        assert_eq!(entry.url, Some("https://mcp.example.com/sse".to_string()));
        assert!(entry.headers.is_some());
    }

    // =========================================================================
    // HTTP client tests with wiremock
    // =========================================================================

    #[tokio::test]
    async fn test_search_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers"))
            .and(query_param("search", "filesystem"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "servers": [
                    {
                        "server": {
                            "name": "io.github.test/filesystem-mcp",
                            "description": "A filesystem MCP server",
                            "packages": [{
                                "registryType": "npm",
                                "identifier": "@mcp/filesystem",
                                "name": "filesystem"
                            }]
                        }
                    }
                ],
                "metadata": {
                    "count": 1
                }
            })))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let servers = client.search("filesystem", 10).await.unwrap();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "io.github.test/filesystem-mcp");
        assert_eq!(
            servers[0].description,
            Some("A filesystem MCP server".to_string())
        );
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "servers": [],
                "metadata": {
                    "count": 0
                }
            })))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let servers = client.search("nonexistent", 10).await.unwrap();

        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_search_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let result = client.search("test", 10).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[tokio::test]
    async fn test_list_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "servers": [
                    {
                        "server": {
                            "name": "server1",
                            "description": "First server"
                        }
                    },
                    {
                        "server": {
                            "name": "server2",
                            "description": "Second server"
                        }
                    }
                ],
                "metadata": {
                    "count": 2,
                    "nextCursor": "cursor123"
                }
            })))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let (servers, next_cursor) = client.list(10, None).await.unwrap();

        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "server1");
        assert_eq!(servers[1].name, "server2");
        assert_eq!(next_cursor, Some("cursor123".to_string()));
    }

    #[tokio::test]
    async fn test_list_no_next_cursor() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "servers": [
                    {
                        "server": {
                            "name": "last-server"
                        }
                    }
                ],
                "metadata": {
                    "count": 1
                }
            })))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let (servers, next_cursor) = client.list(10, None).await.unwrap();

        assert_eq!(servers.len(), 1);
        assert!(next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_get_server_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers/io.github.test/my-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server": {
                    "name": "io.github.test/my-server",
                    "description": "My test server",
                    "version": "1.0.0",
                    "packages": [{
                        "registryType": "npm",
                        "identifier": "@test/my-server"
                    }],
                    "repository": {
                        "url": "https://github.com/test/my-server"
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let server = client.get_server("io.github.test/my-server").await.unwrap();

        assert_eq!(server.name, "io.github.test/my-server");
        assert_eq!(server.description, Some("My test server".to_string()));
        assert_eq!(server.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_get_server_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v0/servers/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let client = RegistryClient::with_base_url(mock_server.uri());
        let result = client.get_server("nonexistent").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Server not found"));
    }

    #[tokio::test]
    async fn test_with_base_url_constructor() {
        let client = RegistryClient::with_base_url("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_default_base_url() {
        let client = RegistryClient::new();
        assert_eq!(client.base_url, "https://registry.modelcontextprotocol.io");
    }

    // =========================================================================
    // Conversion logic tests
    // =========================================================================

    #[test]
    fn test_pypi_package_conversion() {
        let server = RegistryServer {
            name: "io.github.test/python-mcp".to_string(),
            description: Some("A Python MCP server".to_string()),
            repository: None,
            version: Some("2.0.0".to_string()),
            packages: Some(vec![Package {
                registry_type: "pypi".to_string(),
                identifier: Some("python-mcp-server".to_string()),
                name: None,
                version: None,
                arguments: None,
                environment_variables: Some(vec![EnvironmentVariable {
                    name: "API_KEY".to_string(),
                    description: Some("API key for authentication".to_string()),
                    is_required: Some(true),
                    is_secret: Some(true),
                    default: None,
                    format: None,
                    extra: None,
                }]),
                transport: None,
                extra: None,
            }]),
            remotes: None,
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "python-mcp");
        assert_eq!(entry.mcp_type, "stdio");
        assert_eq!(entry.command, Some("uvx".to_string()));
        assert_eq!(entry.args, Some(vec!["python-mcp-server".to_string()]));
        assert!(entry.env_placeholders.is_some());
        let placeholders = entry.env_placeholders.unwrap();
        assert_eq!(placeholders[0].name, "API_KEY");
        assert!(placeholders[0].is_required);
    }

    #[test]
    fn test_docker_package_conversion() {
        let server = RegistryServer {
            name: "io.github.test/docker-mcp".to_string(),
            description: Some("A Docker MCP server".to_string()),
            repository: None,
            version: None,
            packages: Some(vec![Package {
                registry_type: "oci".to_string(),
                identifier: Some("docker.io/user/mcp-server:latest".to_string()),
                name: None,
                version: None,
                arguments: None,
                environment_variables: None,
                transport: None,
                extra: None,
            }]),
            remotes: None,
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "docker-mcp");
        assert_eq!(entry.mcp_type, "stdio");
        assert_eq!(entry.command, Some("docker".to_string()));
        let args = entry.args.unwrap();
        assert_eq!(args[0], "run");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], "--rm");
        assert_eq!(args[3], "user/mcp-server"); // Version tag stripped
    }

    #[test]
    fn test_http_remote_conversion() {
        let server = RegistryServer {
            name: "http-server".to_string(),
            description: Some("An HTTP server".to_string()),
            repository: None,
            version: None,
            packages: None,
            remotes: Some(vec![Remote {
                transport_type: "http".to_string(),
                url: "https://api.example.com/mcp".to_string(),
                headers: None,
                extra: None,
            }]),
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "http-server");
        assert_eq!(entry.mcp_type, "http");
        assert_eq!(entry.url, Some("https://api.example.com/mcp".to_string()));
        assert!(entry.command.is_none());
    }

    #[test]
    fn test_streamable_http_remote_conversion() {
        let server = RegistryServer {
            name: "streamable-server".to_string(),
            description: None,
            repository: None,
            version: None,
            packages: None,
            remotes: Some(vec![Remote {
                transport_type: "streamable-http".to_string(),
                url: "https://stream.example.com".to_string(),
                headers: None,
                extra: None,
            }]),
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();
        assert_eq!(entry.mcp_type, "http"); // streamable-http maps to http
    }

    #[test]
    fn test_server_no_packages_or_remotes() {
        let server = RegistryServer {
            name: "empty-server".to_string(),
            description: None,
            repository: None,
            version: None,
            packages: None,
            remotes: None,
            updated_at: None,
            extra: None,
        };

        let result = server.to_mcp_entry();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no packages or remotes"));
    }

    #[test]
    fn test_unsupported_registry_type_falls_back_to_remote() {
        let server = RegistryServer {
            name: "hybrid-server".to_string(),
            description: Some("Server with unsupported package type".to_string()),
            repository: None,
            version: None,
            packages: Some(vec![Package {
                registry_type: "unknown-type".to_string(),
                identifier: Some("some-package".to_string()),
                name: None,
                version: None,
                arguments: None,
                environment_variables: None,
                transport: None,
                extra: None,
            }]),
            remotes: Some(vec![Remote {
                transport_type: "sse".to_string(),
                url: "https://fallback.example.com/sse".to_string(),
                headers: None,
                extra: None,
            }]),
            updated_at: None,
            extra: None,
        };

        // Should fall back to remote since package type is unsupported
        let entry = server.to_mcp_entry().unwrap();
        assert_eq!(entry.mcp_type, "sse");
        assert_eq!(
            entry.url,
            Some("https://fallback.example.com/sse".to_string())
        );
    }

    #[test]
    fn test_remote_with_header_placeholder() {
        let server = RegistryServer {
            name: "auth-server".to_string(),
            description: None,
            repository: None,
            version: None,
            packages: None,
            remotes: Some(vec![Remote {
                transport_type: "sse".to_string(),
                url: "https://auth.example.com".to_string(),
                headers: Some(vec![RemoteHeader {
                    name: "X-API-Key".to_string(),
                    value: None, // No value provided - should create placeholder
                    description: Some("Your API key".to_string()),
                    is_required: Some(true),
                    is_secret: Some(true),
                }]),
                extra: None,
            }]),
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();
        let headers = entry.headers.unwrap();
        // When no value is provided, it should use a placeholder format
        assert_eq!(headers.get("X-API-Key"), Some(&"${X-API-Key}".to_string()));
    }

    #[test]
    fn test_extract_package_name_docker() {
        assert_eq!(
            extract_package_name("docker.io/user/image:v1.0", "oci"),
            "user/image"
        );
        assert_eq!(
            extract_package_name("ghcr.io/org/image:latest", "docker"),
            "org/image"
        );
        assert_eq!(
            extract_package_name("gcr.io/project/image", "oci"),
            "project/image"
        );
    }

    #[test]
    fn test_extract_package_name_npm() {
        // Non-docker types should pass through unchanged
        assert_eq!(
            extract_package_name("@modelcontextprotocol/server-filesystem", "npm"),
            "@modelcontextprotocol/server-filesystem"
        );
    }

    #[test]
    fn test_package_with_named_arguments() {
        let server = RegistryServer {
            name: "args-server".to_string(),
            description: None,
            repository: None,
            version: None,
            packages: Some(vec![Package {
                registry_type: "npm".to_string(),
                identifier: Some("@test/server".to_string()),
                name: None,
                version: None,
                arguments: Some(vec![PackageArgument {
                    arg_type: Some("named".to_string()),
                    name: Some("--config".to_string()),
                    value: Some("./config.json".to_string()),
                    description: None,
                    is_required: None,
                    default: None,
                }]),
                environment_variables: None,
                transport: None,
                extra: None,
            }]),
            remotes: None,
            updated_at: None,
            extra: None,
        };

        let entry = server.to_mcp_entry().unwrap();
        let args = entry.args.unwrap();
        assert!(args.contains(&"--config".to_string()));
        assert!(args.contains(&"./config.json".to_string()));
    }
}
