use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::db::models::{CreateMcpRequest, CreateSkillRequest};
use crate::db::Database;
use crate::services::memory_writer::{self, MemoryScope};

const GITHUB_API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "claude-code-tool-manager/1.0";
const GIST_DESCRIPTION: &str = "claude-code-tool-manager-sync";
const SCHEMA_VERSION: u32 = 1;

// ─── Data Structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMeta {
    pub machine_id: String,
    pub last_synced_at: String,
    pub project_mappings: Vec<ProjectMapping>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMapping {
    pub local_path: String,
    pub canonical_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSyncEntry {
    pub name: String,
    pub description: Option<String>,
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSyncEntry {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
    pub context: Option<String>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSyncEntry {
    pub canonical_name: String,
    pub claude_md_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayload {
    pub global_claude_md: Option<String>,
    pub mcps: Option<Vec<McpSyncEntry>>,
    pub skills: Option<Vec<SkillSyncEntry>>,
    pub projects: Vec<ProjectSyncEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub sync_global_claude_md: bool,
    pub sync_skills: bool,
    pub sync_mcps: bool,
    pub sync_project_claude_mds: Vec<String>, // project IDs
    pub auto_sync_on_launch: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_global_claude_md: true,
            sync_skills: false,
            sync_mcps: false,
            sync_project_claude_mds: Vec::new(),
            auto_sync_on_launch: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAuthStatus {
    pub is_authenticated: bool,
    pub username: Option<String>,
    pub has_gh_cli: bool,
    pub gist_id: Option<String>,
    pub gist_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub last_pushed_at: Option<String>,
    pub last_pulled_at: Option<String>,
    pub gist_id: Option<String>,
    pub gist_url: Option<String>,
    pub item_counts: SyncItemCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncItemCounts {
    pub mcps: usize,
    pub skills: usize,
    pub projects: usize,
    pub has_global_claude_md: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub pushed: Vec<String>,
    pub pulled: Vec<String>,
    pub conflicts: Vec<String>,
    pub synced_at: String,
    pub gist_url: String,
}

// ─── Gist API response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GistFile {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GistResponse {
    id: String,
    html_url: String,
    description: Option<String>,
    files: HashMap<String, GistFile>,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

// ─── Service ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GistSyncService {
    client: Client,
    api_base: String,
}

impl GistSyncService {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_base: GITHUB_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(api_base: String) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, api_base }
    }

    fn auth_headers(&self, token: &str) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        if let Ok(auth_value) = header::HeaderValue::from_str(&format!("Bearer {}", token)) {
            headers.insert(header::AUTHORIZATION, auth_value);
        }
        headers
    }

    /// Get the authenticated GitHub username
    pub async fn get_authenticated_user(&self, token: &str) -> Result<String> {
        let url = format!("{}/user", self.api_base);
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }

        let user: GitHubUser = response.json().await?;
        Ok(user.login)
    }

    /// Verify the token carries the `gist` OAuth scope. Classic tokens (what
    /// `gh auth token` returns) advertise granted scopes in the
    /// `X-OAuth-Scopes` response header. Fine-grained PATs don't set this
    /// header — in that case we can't verify upfront and fall through.
    ///
    /// Returns a user-facing error with the exact remediation command when
    /// the scope is confirmed absent.
    pub async fn verify_gist_scope(&self, token: &str) -> Result<()> {
        let url = format!("{}/user", self.api_base);
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, text));
        }

        let Some(scopes_header) = response.headers().get("x-oauth-scopes") else {
            // Fine-grained PAT or similar — can't verify, defer to API-level
            // errors on the actual gist call.
            return Ok(());
        };

        let scopes_str = scopes_header.to_str().unwrap_or("");
        let has_gist = scopes_str.split(',').map(|s| s.trim()).any(|s| s == "gist");

        if has_gist {
            Ok(())
        } else {
            Err(anyhow!(
                "GitHub token is missing the 'gist' scope. Run: gh auth refresh -h github.com -s gist"
            ))
        }
    }

    /// Find existing sync gist or create a new one
    pub async fn find_or_create_gist(&self, token: &str) -> Result<(String, String)> {
        // Search user's gists for our sync gist (paginated — GitHub caps at 100 per page)
        let mut page = 1u32;
        loop {
            let url = format!("{}/gists?per_page=100&page={}", self.api_base, page);
            let response = self
                .client
                .get(&url)
                .headers(self.auth_headers(token))
                .send()
                .await?;

            if !response.status().is_success() {
                break;
            }

            let gists: Vec<GistResponse> = response.json().await?;
            if gists.is_empty() {
                break;
            }

            for gist in &gists {
                if gist.description.as_deref() == Some(GIST_DESCRIPTION) {
                    return Ok((gist.id.clone(), gist.html_url.clone()));
                }
            }

            page += 1;
        }

        // Not found — create a new private gist
        let mut files = HashMap::new();
        files.insert(
            "_meta.json".to_string(),
            serde_json::json!({ "content": serde_json::to_string_pretty(&SyncMeta {
                machine_id: get_machine_id(),
                last_synced_at: now_iso(),
                project_mappings: Vec::new(),
                schema_version: SCHEMA_VERSION,
            })? }),
        );

        let body = serde_json::json!({
            "description": GIST_DESCRIPTION,
            "public": false,
            "files": files,
        });

        let url = format!("{}/gists", self.api_base);
        let response = self
            .client
            .post(&url)
            .headers(self.auth_headers(token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create gist: {} {}", status, text));
        }

        let gist: GistResponse = response.json().await?;
        Ok((gist.id, gist.html_url))
    }

    /// Push local state to the gist
    pub async fn push(
        &self,
        token: &str,
        gist_id: &str,
        payload: &SyncPayload,
        meta: &SyncMeta,
    ) -> Result<SyncResult> {
        let mut files: HashMap<String, serde_json::Value> = HashMap::new();
        let mut pushed = Vec::new();

        // Meta
        files.insert(
            "_meta.json".to_string(),
            serde_json::json!({ "content": serde_json::to_string_pretty(meta)? }),
        );

        // Global CLAUDE.md
        if let Some(ref content) = payload.global_claude_md {
            files.insert(
                "global_claude_md.md".to_string(),
                serde_json::json!({ "content": content }),
            );
            pushed.push("Global CLAUDE.md".to_string());
        }

        // MCPs
        if let Some(ref mcps) = payload.mcps {
            files.insert(
                "mcps.json".to_string(),
                serde_json::json!({ "content": serde_json::to_string_pretty(mcps)? }),
            );
            pushed.push(format!("{} MCPs", mcps.len()));
        }

        // Skills
        if let Some(ref skills) = payload.skills {
            files.insert(
                "skills.json".to_string(),
                serde_json::json!({ "content": serde_json::to_string_pretty(skills)? }),
            );
            pushed.push(format!("{} Skills", skills.len()));
        }

        // Projects
        if !payload.projects.is_empty() {
            files.insert(
                "projects.json".to_string(),
                serde_json::json!({ "content": serde_json::to_string_pretty(&payload.projects)? }),
            );
            pushed.push(format!("{} Projects", payload.projects.len()));
        }

        let body = serde_json::json!({ "files": files });
        let url = format!("{}/gists/{}", self.api_base, gist_id);
        let response = self
            .client
            .patch(&url)
            .headers(self.auth_headers(token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to update gist: {} {}", status, text));
        }

        let gist: GistResponse = response.json().await?;
        Ok(SyncResult {
            pushed,
            pulled: Vec::new(),
            conflicts: Vec::new(),
            synced_at: now_iso(),
            gist_url: gist.html_url,
        })
    }

    /// Pull remote state from the gist
    pub async fn pull(&self, token: &str, gist_id: &str) -> Result<(SyncPayload, SyncMeta)> {
        let url = format!("{}/gists/{}", self.api_base, gist_id);
        let response = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to fetch gist: {} {}", status, text));
        }

        let gist: GistResponse = response.json().await?;

        // Parse meta
        let meta = if let Some(file) = gist.files.get("_meta.json") {
            let content = file.content.as_deref().unwrap_or("{}");
            serde_json::from_str(content).unwrap_or(SyncMeta {
                machine_id: String::new(),
                last_synced_at: String::new(),
                project_mappings: Vec::new(),
                schema_version: SCHEMA_VERSION,
            })
        } else {
            SyncMeta {
                machine_id: String::new(),
                last_synced_at: String::new(),
                project_mappings: Vec::new(),
                schema_version: SCHEMA_VERSION,
            }
        };

        // Parse global CLAUDE.md
        let global_claude_md = gist
            .files
            .get("global_claude_md.md")
            .and_then(|f| f.content.clone());

        // Parse MCPs
        let mcps = gist
            .files
            .get("mcps.json")
            .and_then(|f| f.content.as_deref())
            .and_then(|c| serde_json::from_str(c).ok());

        // Parse Skills
        let skills = gist
            .files
            .get("skills.json")
            .and_then(|f| f.content.as_deref())
            .and_then(|c| serde_json::from_str(c).ok());

        // Parse Projects
        let projects = gist
            .files
            .get("projects.json")
            .and_then(|f| f.content.as_deref())
            .and_then(|c| serde_json::from_str(c).ok())
            .unwrap_or_default();

        Ok((
            SyncPayload {
                global_claude_md,
                mcps,
                skills,
                projects,
            },
            meta,
        ))
    }
}

// ─── Payload Builders ───────────────────────────────────────────────────────

/// Build a SyncPayload from local state
pub fn build_local_payload(db: &Database, config: &SyncConfig) -> Result<SyncPayload> {
    let mut payload = SyncPayload {
        global_claude_md: None,
        mcps: None,
        skills: None,
        projects: Vec::new(),
    };

    // Global CLAUDE.md
    if config.sync_global_claude_md {
        let path = memory_writer::resolve_memory_path(&MemoryScope::User, None)?;
        if path.exists() {
            payload.global_claude_md = Some(std::fs::read_to_string(&path)?);
        }
    }

    // MCPs
    if config.sync_mcps {
        let all_mcps = db.get_all_mcps()?;
        let entries: Vec<McpSyncEntry> = all_mcps
            .iter()
            .filter(|m| m.source != "system") // Skip system MCPs
            .map(|m| McpSyncEntry {
                name: m.name.clone(),
                description: m.description.clone(),
                mcp_type: m.mcp_type.clone(),
                command: m.command.clone(),
                args: m.args.clone(),
                url: m.url.clone(),
                headers: None, // Excluded from sync — may contain secrets
                env: None,     // Excluded from sync — may contain API keys
                icon: m.icon.clone(),
                tags: m.tags.clone(),
            })
            .collect();
        if !entries.is_empty() {
            payload.mcps = Some(entries);
        }
    }

    // Skills
    if config.sync_skills {
        let all_skills = db.get_all_skills()?;
        let entries: Vec<SkillSyncEntry> = all_skills
            .iter()
            .filter(|s| s.source != "system")
            .map(|s| SkillSyncEntry {
                name: s.name.clone(),
                description: s.description.clone(),
                content: s.content.clone(),
                allowed_tools: s.allowed_tools.clone(),
                model: s.model.clone(),
                tags: s.tags.clone(),
                context: s.context.clone(),
                agent: s.agent.clone(),
            })
            .collect();
        if !entries.is_empty() {
            payload.skills = Some(entries);
        }
    }

    // Project CLAUDE.md files
    if !config.sync_project_claude_mds.is_empty() {
        let all_projects = db.get_all_projects()?;
        for project in &all_projects {
            let id_str = project.id.to_string();
            if config.sync_project_claude_mds.contains(&id_str) {
                let project_path = Path::new(&project.path);
                if let Ok(claude_md_path) =
                    memory_writer::resolve_memory_path(&MemoryScope::Project, Some(project_path))
                {
                    let content = if claude_md_path.exists() {
                        Some(std::fs::read_to_string(&claude_md_path)?)
                    } else {
                        None
                    };
                    // Use the project directory name as canonical name
                    let canonical = project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.name.clone());
                    payload.projects.push(ProjectSyncEntry {
                        canonical_name: canonical,
                        claude_md_content: content,
                    });
                }
            }
        }
    }

    Ok(payload)
}

/// Apply a pulled payload to local state
pub fn apply_pulled_payload(
    db: &Database,
    payload: &SyncPayload,
    config: &SyncConfig,
    mappings: &[ProjectMapping],
) -> Result<SyncResult> {
    let mut pulled = Vec::new();
    let mut conflicts = Vec::new();

    // Global CLAUDE.md
    if config.sync_global_claude_md {
        if let Some(ref content) = payload.global_claude_md {
            let path = memory_writer::resolve_memory_path(&MemoryScope::User, None)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Back up existing file before overwriting
            if path.exists() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let backup = path.with_file_name(format!("{}.bak", file_name));
                let _ = std::fs::copy(&path, &backup);
            }
            std::fs::write(&path, content)?;
            pulled.push("Global CLAUDE.md".to_string());
        }
    }

    // MCPs — upsert by name
    if config.sync_mcps {
        if let Some(ref mcps) = payload.mcps {
            let mut count = 0;
            for entry in mcps {
                match db.get_mcp_by_name(&entry.name) {
                    Ok(Some(existing)) => {
                        let mut updated = existing.clone();
                        updated.description = entry.description.clone();
                        updated.mcp_type = entry.mcp_type.clone();
                        updated.command = entry.command.clone();
                        updated.args = entry.args.clone();
                        updated.url = entry.url.clone();
                        updated.headers = entry.headers.clone();
                        updated.env = entry.env.clone();
                        updated.icon = entry.icon.clone();
                        updated.tags = entry.tags.clone();
                        if let Err(e) = db.update_mcp(&updated) {
                            conflicts.push(format!("MCP '{}': {}", entry.name, e));
                            continue;
                        }
                    }
                    Ok(None) => {
                        let req = CreateMcpRequest {
                            name: entry.name.clone(),
                            description: entry.description.clone(),
                            mcp_type: entry.mcp_type.clone(),
                            command: entry.command.clone(),
                            args: entry.args.clone(),
                            url: entry.url.clone(),
                            headers: entry.headers.clone(),
                            env: entry.env.clone(),
                            icon: entry.icon.clone(),
                            tags: entry.tags.clone(),
                        };
                        if let Err(e) = db.create_mcp(&req) {
                            conflicts.push(format!("MCP '{}': {}", entry.name, e));
                            continue;
                        }
                    }
                    Err(e) => {
                        conflicts.push(format!("MCP '{}': {}", entry.name, e));
                        continue;
                    }
                }
                count += 1;
            }
            if count > 0 {
                pulled.push(format!("{} MCPs", count));
            }
        }
    }

    // Skills — upsert by name (find existing via get_all_skills)
    if config.sync_skills {
        if let Some(ref skills) = payload.skills {
            let existing_skills = db.get_all_skills().unwrap_or_default();
            let mut count = 0;
            for entry in skills {
                let existing = existing_skills.iter().find(|s| s.name == entry.name);
                if let Some(existing) = existing {
                    // Delete and recreate (no update_skill in DB layer)
                    let _ = db.delete_skill(existing.id);
                }
                let req = CreateSkillRequest {
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    content: entry.content.clone(),
                    allowed_tools: entry.allowed_tools.clone(),
                    model: entry.model.clone(),
                    disable_model_invocation: None,
                    tags: entry.tags.clone(),
                    context: entry.context.clone(),
                    agent: entry.agent.clone(),
                    hooks: None,
                    paths: None,
                    shell: None,
                    once: None,
                    effort: None,
                };
                match db.create_skill(&req) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        conflicts.push(format!("Skill '{}': {}", entry.name, e));
                    }
                }
            }
            if count > 0 {
                pulled.push(format!("{} Skills", count));
            }
        }
    }

    // Project CLAUDE.md files
    for project_entry in &payload.projects {
        if let Some(ref content) = project_entry.claude_md_content {
            // Find local path via mapping
            let local_path = mappings
                .iter()
                .find(|m| m.canonical_name == project_entry.canonical_name)
                .map(|m| m.local_path.clone());

            if let Some(path_str) = local_path {
                let project_path = Path::new(&path_str);
                if project_path.exists() {
                    match memory_writer::resolve_memory_path(
                        &MemoryScope::Project,
                        Some(project_path),
                    ) {
                        Ok(claude_md_path) => {
                            if let Some(parent) = claude_md_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            // Back up existing file before overwriting
                            if claude_md_path.exists() {
                                let file_name = claude_md_path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy();
                                let backup =
                                    claude_md_path.with_file_name(format!("{}.bak", file_name));
                                let _ = std::fs::copy(&claude_md_path, &backup);
                            }
                            if let Err(e) = std::fs::write(&claude_md_path, content) {
                                conflicts.push(format!(
                                    "Project '{}': {}",
                                    project_entry.canonical_name, e
                                ));
                            } else {
                                pulled.push(format!("Project '{}'", project_entry.canonical_name));
                            }
                        }
                        Err(e) => {
                            conflicts
                                .push(format!("Project '{}': {}", project_entry.canonical_name, e));
                        }
                    }
                } else {
                    conflicts.push(format!(
                        "Project '{}': local path '{}' not found",
                        project_entry.canonical_name, path_str
                    ));
                }
            } else {
                conflicts.push(format!(
                    "Project '{}': no local path mapping configured",
                    project_entry.canonical_name
                ));
            }
        }
    }

    Ok(SyncResult {
        pushed: Vec::new(),
        pulled,
        conflicts,
        synced_at: now_iso(),
        gist_url: String::new(), // caller fills in
    })
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn get_machine_id() -> String {
    // Use hostname as machine identifier
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_authenticated_user() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "login": "testuser"
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let user = service.get_authenticated_user("test-token").await.unwrap();
        assert_eq!(user, "testuser");
    }

    #[tokio::test]
    async fn test_get_authenticated_user_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Bad credentials"
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let result = service.get_authenticated_user("bad-token").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("401"));
    }

    #[tokio::test]
    async fn test_verify_gist_scope_present() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("X-OAuth-Scopes", "repo, gist, read:org")
                    .set_body_json(serde_json::json!({ "login": "testuser" })),
            )
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let result = service.verify_gist_scope("test-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_gist_scope_missing() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("X-OAuth-Scopes", "repo, read:org")
                    .set_body_json(serde_json::json!({ "login": "testuser" })),
            )
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let err = service.verify_gist_scope("test-token").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("'gist' scope"), "got: {}", msg);
        assert!(msg.contains("gh auth refresh"), "got: {}", msg);
    }

    #[tokio::test]
    async fn test_verify_gist_scope_header_absent() {
        // Fine-grained PATs don't emit X-OAuth-Scopes — we should not block.
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "login": "testuser" })),
            )
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let result = service.verify_gist_scope("test-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_gist_scope_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/user"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Bad credentials"
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let result = service.verify_gist_scope("bad-token").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("401"));
    }

    #[tokio::test]
    async fn test_find_or_create_gist_finds_existing() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "abc123",
                    "html_url": "https://gist.github.com/abc123",
                    "description": "claude-code-tool-manager-sync",
                    "files": {}
                }
            ])))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let (id, url) = service.find_or_create_gist("test-token").await.unwrap();
        assert_eq!(id, "abc123");
        assert_eq!(url, "https://gist.github.com/abc123");
    }

    #[tokio::test]
    async fn test_find_or_create_gist_creates_new() {
        let mock_server = MockServer::start().await;

        // No existing gists
        Mock::given(method("GET"))
            .and(path("/gists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        // Create new
        Mock::given(method("POST"))
            .and(path("/gists"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": "new123",
                "html_url": "https://gist.github.com/new123",
                "description": "claude-code-tool-manager-sync",
                "files": {}
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let (id, _) = service.find_or_create_gist("test-token").await.unwrap();
        assert_eq!(id, "new123");
    }

    #[tokio::test]
    async fn test_push_payload() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/gists/gist123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "gist123",
                "html_url": "https://gist.github.com/gist123",
                "description": "claude-code-tool-manager-sync",
                "files": {}
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let payload = SyncPayload {
            global_claude_md: Some("# My Config".to_string()),
            mcps: Some(vec![McpSyncEntry {
                name: "test-mcp".to_string(),
                description: None,
                mcp_type: "stdio".to_string(),
                command: Some("npx".to_string()),
                args: None,
                url: None,
                headers: None,
                env: None,
                icon: None,
                tags: None,
            }]),
            skills: None,
            projects: Vec::new(),
        };
        let meta = SyncMeta {
            machine_id: "test-machine".to_string(),
            last_synced_at: now_iso(),
            project_mappings: Vec::new(),
            schema_version: SCHEMA_VERSION,
        };

        let result = service
            .push("test-token", "gist123", &payload, &meta)
            .await
            .unwrap();
        assert!(result.pushed.contains(&"Global CLAUDE.md".to_string()));
        assert!(result.pushed.iter().any(|p| p.contains("MCP")));
    }

    #[tokio::test]
    async fn test_pull_payload() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gists/gist123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "gist123",
                "html_url": "https://gist.github.com/gist123",
                "description": "claude-code-tool-manager-sync",
                "files": {
                    "_meta.json": { "content": "{\"machineId\":\"m1\",\"lastSyncedAt\":\"2024-01-01\",\"projectMappings\":[],\"schemaVersion\":1}" },
                    "global_claude_md.md": { "content": "# Synced Config" },
                    "mcps.json": { "content": "[{\"name\":\"test\",\"mcpType\":\"stdio\"}]" },
                    "skills.json": { "content": "[{\"name\":\"my-skill\",\"content\":\"hello\"}]" }
                }
            })))
            .mount(&mock_server)
            .await;

        let service = GistSyncService::with_base_url(mock_server.uri());
        let (payload, meta) = service.pull("test-token", "gist123").await.unwrap();

        assert_eq!(
            payload.global_claude_md,
            Some("# Synced Config".to_string())
        );
        assert!(payload.mcps.is_some());
        assert!(payload.skills.is_some());
        assert_eq!(meta.machine_id, "m1");
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.sync_global_claude_md);
        assert!(!config.sync_skills);
        assert!(!config.sync_mcps);
        assert!(config.sync_project_claude_mds.is_empty());
        assert!(!config.auto_sync_on_launch);
    }

    #[test]
    fn test_sync_config_serialization() {
        let config = SyncConfig {
            sync_global_claude_md: true,
            sync_skills: true,
            sync_mcps: false,
            sync_project_claude_mds: vec!["1".to_string(), "2".to_string()],
            auto_sync_on_launch: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SyncConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.sync_skills);
        assert_eq!(deserialized.sync_project_claude_mds.len(), 2);
    }

    #[test]
    fn test_sync_payload_serialization() {
        let payload = SyncPayload {
            global_claude_md: Some("test".to_string()),
            mcps: None,
            skills: None,
            projects: vec![ProjectSyncEntry {
                canonical_name: "my-project".to_string(),
                claude_md_content: Some("# Project".to_string()),
            }],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("my-project"));
    }

    #[test]
    fn test_get_machine_id() {
        let id = get_machine_id();
        assert!(!id.is_empty());
    }
}
