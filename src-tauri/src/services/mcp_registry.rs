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
pub struct RegistryResponse {
    pub servers: Vec<RegistryServer>,
    pub metadata: Option<RegistryMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct RegistryMetadata {
    pub count: Option<u32>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub repository: Option<RepositoryInfo>,
    pub version_detail: Option<VersionDetail>,
    pub packages: Option<Vec<Package>>,
    pub remotes: Option<Vec<Remote>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryInfo {
    pub url: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDetail {
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub registry_type: String, // "npm", "pypi", "nuget", "oci", "mcpb"
    pub name: Option<String>,
    pub version: Option<String>,
    pub package_arguments: Option<Vec<PackageArgument>>,
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageArgument {
    #[serde(rename = "type")]
    pub arg_type: Option<String>, // "positional" or "named"
    pub name: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub is_required: Option<bool>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub description: Option<String>,
    pub is_required: Option<bool>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Remote {
    #[serde(rename = "type")]
    pub transport_type: String, // "sse" or "http"
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
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
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Search for MCP servers in the registry
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<RegistryServer>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/v0/servers?search={}&limit={}&status=active",
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

        let registry_response: RegistryResponse = response.json().await?;
        Ok(registry_response.servers)
    }

    /// List servers with pagination
    pub async fn list(&self, limit: u32, cursor: Option<&str>) -> Result<(Vec<RegistryServer>, Option<String>)> {
        let mut url = format!(
            "{}/v0/servers?limit={}&status=active",
            REGISTRY_BASE_URL, limit
        );

        if let Some(c) = cursor {
            url.push_str(&format!("&cursor={}", urlencoding::encode(c)));
        }

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

        let registry_response: RegistryResponse = response.json().await?;
        let next_cursor = registry_response.metadata.and_then(|m| m.next_cursor);

        Ok((registry_response.servers, next_cursor))
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

        let server: RegistryServer = response.json().await?;
        Ok(server)
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
        // Try packages first (local/stdio MCPs), then remotes (SSE/HTTP)
        if let Some(packages) = &self.packages {
            if let Some(package) = packages.first() {
                return package_to_mcp_entry(self, package);
            }
        }

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
    let (command, mut args) = match package.registry_type.as_str() {
        "npm" => {
            let pkg_name = package
                .name
                .as_ref()
                .ok_or_else(|| anyhow!("Missing package.name for npm package"))?;
            ("npx".to_string(), vec!["-y".to_string(), pkg_name.clone()])
        }
        "pypi" => {
            let pkg_name = package
                .name
                .as_ref()
                .ok_or_else(|| anyhow!("Missing package.name for pypi package"))?;
            ("uvx".to_string(), vec![pkg_name.clone()])
        }
        "oci" | "docker" => {
            let pkg_name = package
                .name
                .as_ref()
                .ok_or_else(|| anyhow!("Missing package.name for docker package"))?;
            (
                "docker".to_string(),
                vec!["run".to_string(), "-i".to_string(), "--rm".to_string(), pkg_name.clone()],
            )
        }
        other => {
            return Err(anyhow!("Unsupported registry type: {}", other));
        }
    };

    // Add package arguments
    if let Some(pkg_args) = &package.package_arguments {
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
        registry_id: server.id.clone(),
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
        version: server.version_detail.as_ref().and_then(|v| v.version.clone()),
        registry_type: Some(package.registry_type.clone()),
    })
}

fn remote_to_mcp_entry(server: &RegistryServer, remote: &Remote) -> RegistryMcpEntry {
    let mcp_type = match remote.transport_type.as_str() {
        "sse" => "sse",
        _ => "http",
    };

    RegistryMcpEntry {
        registry_id: server.id.clone(),
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
        version: server.version_detail.as_ref().and_then(|v| v.version.clone()),
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
            id: "test-id".to_string(),
            name: "io.github.test/filesystem-server".to_string(),
            description: Some("A filesystem MCP server".to_string()),
            repository: Some(RepositoryInfo {
                url: Some("https://github.com/test/filesystem".to_string()),
                source: None,
            }),
            version_detail: Some(VersionDetail {
                version: Some("1.0.0".to_string()),
            }),
            packages: Some(vec![Package {
                registry_type: "npm".to_string(),
                name: Some("@modelcontextprotocol/server-filesystem".to_string()),
                version: Some("1.0.0".to_string()),
                package_arguments: Some(vec![PackageArgument {
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
            id: "test-id".to_string(),
            name: "remote-server".to_string(),
            description: Some("A remote SSE server".to_string()),
            repository: None,
            version_detail: None,
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
