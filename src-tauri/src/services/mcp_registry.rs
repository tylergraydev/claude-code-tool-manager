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
    pub registry_type: String, // "npm", "pypi", "nuget", "oci", "mcpb"
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
    pub headers: Option<HashMap<String, String>>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: Option<HashMap<String, serde_json::Value>>,
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
        }
    }

    /// Search for MCP servers in the registry
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<RegistryServer>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/v0/servers?search={}&limit={}&status=active&version=latest",
            REGISTRY_BASE_URL, encoded_query, limit
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
    pub async fn list(&self, limit: u32, cursor: Option<&str>) -> Result<(Vec<RegistryServer>, Option<String>)> {
        let mut url = format!(
            "{}/v0/servers?limit={}&status=active&version=latest",
            REGISTRY_BASE_URL, limit
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

        log::info!("[Registry] Found {} servers in response", servers_array.len());

        // Parse each server dynamically (API already filters for latest versions via version=latest param)
        let mut servers = Vec::new();
        for item in servers_array {
            // Each item has { server: {...}, _meta: {...} }
            if let Some(server_obj) = item.get("server") {
                match serde_json::from_value::<RegistryServer>(server_obj.clone()) {
                    Ok(server) => servers.push(server),
                    Err(e) => {
                        let name = server_obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        log::warn!("[Registry] Failed to parse server '{}': {}", name, e);
                        // Continue with other servers instead of failing completely
                    }
                }
            }
        }

        log::info!("[Registry] Parsed {} servers", servers.len());

        Ok((servers, next_cursor))
    }

    /// Get a specific server by ID
    pub async fn get_server(&self, server_id: &str) -> Result<RegistryServer> {
        let url = format!("{}/v0/servers/{}", REGISTRY_BASE_URL, server_id);

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
    full_name
        .split('/')
        .last()
        .unwrap_or(full_name)
        .to_string()
}

fn package_to_mcp_entry(server: &RegistryServer, package: &Package) -> Result<RegistryMcpEntry> {
    // Get package name from identifier or name field
    let pkg_name = package.identifier.as_ref()
        .or(package.name.as_ref())
        .ok_or_else(|| anyhow!("Missing package identifier or name"))?;

    // Extract just the package name from identifier like "docker.io/user/image:tag"
    let clean_pkg_name = extract_package_name(pkg_name, &package.registry_type);

    let (command, mut args) = match package.registry_type.as_str() {
        "npm" => {
            ("npx".to_string(), vec!["-y".to_string(), clean_pkg_name])
        }
        "pypi" => {
            ("uvx".to_string(), vec![clean_pkg_name])
        }
        "oci" | "docker" => {
            (
                "docker".to_string(),
                vec!["run".to_string(), "-i".to_string(), "--rm".to_string(), clean_pkg_name],
            )
        }
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
    let env_placeholders = if env_placeholders.is_empty() { None } else { Some(env_placeholders) };

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
            without_registry.split(':').next().unwrap_or(identifier).to_string()
        }
        _ => identifier.to_string()
    }
}

fn remote_to_mcp_entry(server: &RegistryServer, remote: &Remote) -> RegistryMcpEntry {
    let mcp_type = match remote.transport_type.as_str() {
        "sse" => "sse",
        "streamable-http" | "http" => "http",
        _ => "http",
    };

    RegistryMcpEntry {
        registry_id: server.name.clone(),
        name: extract_short_name(&server.name),
        description: server.description.clone(),
        mcp_type: mcp_type.to_string(),
        command: None,
        args: None,
        url: Some(remote.url.clone()),
        headers: remote.headers.clone(),
        env: None,
        env_placeholders: None,
        source_url: server.repository.as_ref().and_then(|r| r.url.clone()),
        version: server.version.clone(),
        registry_type: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            }]),
            remotes: None,
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
                headers: Some(HashMap::from([
                    ("Authorization".to_string(), "Bearer token".to_string()),
                ])),
            }]),
        };

        let entry = server.to_mcp_entry().unwrap();

        assert_eq!(entry.name, "remote-server");
        assert_eq!(entry.mcp_type, "sse");
        assert_eq!(entry.url, Some("https://mcp.example.com/sse".to_string()));
        assert!(entry.headers.is_some());
    }
}
