use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// ─── Public response structs (serialized to frontend) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListInfo {
    pub dir_path: String,
    pub exists: bool,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub folder_name: String,
    pub inferred_path: String,
    pub session_count: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub models_used: Vec<String>,
    pub tool_usage: HashMap<String, u64>,
    pub earliest_session: Option<String>,
    pub latest_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionListInfo {
    pub project_folder: String,
    pub exists: bool,
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub duration_ms: u64,
    pub user_message_count: u64,
    pub assistant_message_count: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub models_used: Vec<String>,
    pub git_branch: Option<String>,
    pub cwd: Option<String>,
    pub tool_counts: HashMap<String, u64>,
    pub first_user_message: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session_id: String,
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub uuid: Option<String>,
    pub role: String,
    pub timestamp: Option<String>,
    pub model: Option<String>,
    pub content_preview: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub usage: Option<MessageUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
}

// ─── Internal parsing helpers (not sent to frontend) ────────────────────────

/// Minimal record parsed from each JSONL line — use Value for flexible content.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRecord {
    #[serde(rename = "type", default)]
    record_type: Option<String>,
    uuid: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    git_branch: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    message: Option<RawMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMessage {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    content: Option<serde_json::Value>,
    #[serde(default)]
    usage: Option<RawUsage>,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,
}

// ─── Path helpers ───────────────────────────────────────────────────────────

/// Returns `~/.claude/projects/`
pub fn projects_dir() -> PathBuf {
    if let Some(base) = directories::BaseDirs::new() {
        base.home_dir().join(".claude").join("projects")
    } else {
        PathBuf::from("~/.claude/projects")
    }
}

/// Best-effort decode of folder name back to a display path.
/// `C--code-claude-code-tool-manager` → `C:/code/claude-code-tool-manager` (or `C:\...` on Windows)
pub fn decode_project_folder(name: &str) -> String {
    // Replace `--` with the OS path separator. This is a heuristic and won't be perfect
    // for paths with actual double-hyphens.
    let sep = std::path::MAIN_SEPARATOR.to_string();

    // Split on `--` and rejoin with separator
    let parts: Vec<&str> = name.split("--").collect();
    if parts.len() <= 1 {
        return name.to_string();
    }

    // First part is drive letter (e.g., "C"), rest are path segments
    // Reconstruct: C:\code\... or C:/code/...
    let mut path = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // Drive letter or root segment
            path.push_str(part);
            path.push(':');
        } else {
            path.push_str(&sep);
            path.push_str(part);
        }
    }
    path
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Scan all project dirs, parse all sessions, return project summaries with aggregates.
pub fn list_projects() -> Result<ProjectListInfo> {
    let dir = projects_dir();
    list_projects_from_dir(&dir)
}

pub fn list_projects_from_dir(dir: &Path) -> Result<ProjectListInfo> {
    let dir_path = dir.to_string_lossy().to_string();

    if !dir.exists() {
        return Ok(ProjectListInfo {
            dir_path,
            exists: false,
            projects: Vec::new(),
        });
    }

    let mut projects = Vec::new();

    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let inferred_path = decode_project_folder(&folder_name);

        // Parse all sessions in this project folder
        let session_list = list_sessions_from_dir(&path, &folder_name)?;
        if session_list.sessions.is_empty() {
            continue; // Skip folders with no JSONL files
        }

        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cache_read = 0u64;
        let mut total_cache_creation = 0u64;
        let mut models_set = std::collections::HashSet::new();
        let mut tool_usage: HashMap<String, u64> = HashMap::new();
        let mut earliest: Option<String> = None;
        let mut latest: Option<String> = None;

        for s in &session_list.sessions {
            total_input += s.total_input_tokens;
            total_output += s.total_output_tokens;
            total_cache_read += s.total_cache_read_tokens;
            total_cache_creation += s.total_cache_creation_tokens;

            for model in &s.models_used {
                models_set.insert(model.clone());
            }

            for (tool, count) in &s.tool_counts {
                *tool_usage.entry(tool.clone()).or_insert(0) += count;
            }

            if let Some(ref ts) = s.first_timestamp {
                match &earliest {
                    None => earliest = Some(ts.clone()),
                    Some(e) if ts < e => earliest = Some(ts.clone()),
                    _ => {}
                }
            }
            if let Some(ref ts) = s.last_timestamp {
                match &latest {
                    None => latest = Some(ts.clone()),
                    Some(l) if ts > l => latest = Some(ts.clone()),
                    _ => {}
                }
            }
        }

        let mut models_used: Vec<String> = models_set.into_iter().collect();
        models_used.sort();

        projects.push(ProjectSummary {
            folder_name,
            inferred_path,
            session_count: session_list.sessions.len() as u64,
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_cache_read_tokens: total_cache_read,
            total_cache_creation_tokens: total_cache_creation,
            models_used,
            tool_usage,
            earliest_session: earliest,
            latest_session: latest,
        });
    }

    // Sort by latest session descending (most recent first)
    projects.sort_by(|a, b| {
        let a_ts = a.latest_session.as_deref().unwrap_or("");
        let b_ts = b.latest_session.as_deref().unwrap_or("");
        b_ts.cmp(a_ts)
    });

    Ok(ProjectListInfo {
        dir_path,
        exists: true,
        projects,
    })
}

/// Parse all `.jsonl` files in one project dir, return session summaries.
pub fn list_sessions(project_folder: &str) -> Result<SessionListInfo> {
    let dir = projects_dir().join(project_folder);
    list_sessions_from_dir(&dir, project_folder)
}

pub fn list_sessions_from_dir(dir: &Path, project_folder: &str) -> Result<SessionListInfo> {
    if !dir.exists() {
        return Ok(SessionListInfo {
            project_folder: project_folder.to_string(),
            exists: false,
            sessions: Vec::new(),
        });
    }

    let mut sessions = Vec::new();

    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }

        let session_id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        match parse_session_summary(&path, &session_id) {
            Ok(summary) => sessions.push(summary),
            Err(e) => {
                warn!(
                    "[SessionExplorer] Skipping unparseable session {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    // Sort by first timestamp descending (most recent first)
    sessions.sort_by(|a, b| {
        let a_ts = a.first_timestamp.as_deref().unwrap_or("");
        let b_ts = b.first_timestamp.as_deref().unwrap_or("");
        b_ts.cmp(a_ts)
    });

    Ok(SessionListInfo {
        project_folder: project_folder.to_string(),
        exists: true,
        sessions,
    })
}

/// Parse single `.jsonl` file, return full message list (deduplicated by requestId).
pub fn get_session_detail(project_folder: &str, session_id: &str) -> Result<SessionDetail> {
    let path = projects_dir()
        .join(project_folder)
        .join(format!("{}.jsonl", session_id));
    parse_session_detail(&path, session_id)
}

// ─── Internal parsers ───────────────────────────────────────────────────────

/// Parse a JSONL file for summary stats. Skips non-user/assistant types.
/// Groups assistant lines by requestId, takes last per requestId for token counts.
fn parse_session_summary(path: &Path, session_id: &str) -> Result<SessionSummary> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut first_timestamp: Option<String> = None;
    let mut last_timestamp: Option<String> = None;
    let mut user_count = 0u64;
    let mut assistant_count = 0u64; // unique requestIds
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut cache_read_tokens = 0u64;
    let mut cache_creation_tokens = 0u64;
    let mut models_set = std::collections::HashSet::new();
    let mut git_branch: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut version: Option<String> = None;
    let mut tool_counts: HashMap<String, u64> = HashMap::new();
    let mut first_user_message: Option<String> = None;

    // Track assistant lines by requestId — keep last usage per requestId
    let mut assistant_request_ids: HashMap<String, Option<RawUsage>> = HashMap::new();
    // Track tool calls from all assistant lines
    let mut seen_tool_calls: HashMap<String, HashMap<String, u64>> = HashMap::new(); // requestId -> tool_counts

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.is_empty() {
            continue;
        }

        // Quick-reject non-interesting record types without full deserialization
        if line.contains("\"type\":\"file-history-snapshot\"")
            || line.contains("\"type\":\"progress\"")
            || line.contains("\"type\":\"bash_progress\"")
            || line.contains("\"type\":\"summary\"")
        {
            continue;
        }

        let record: RawRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let record_type = record.record_type.as_deref().unwrap_or("");

        // Skip types we don't care about
        match record_type {
            "user" | "assistant" | "tool_result" => {}
            _ => continue,
        }

        // Track timestamps
        if let Some(ref ts) = record.timestamp {
            match &first_timestamp {
                None => first_timestamp = Some(ts.clone()),
                Some(ft) if ts < ft => first_timestamp = Some(ts.clone()),
                _ => {}
            }
            match &last_timestamp {
                None => last_timestamp = Some(ts.clone()),
                Some(lt) if ts > lt => last_timestamp = Some(ts.clone()),
                _ => {}
            }
        }

        // Pick up metadata from first record that has it
        if git_branch.is_none() {
            git_branch = record.git_branch;
        }
        if cwd.is_none() {
            cwd = record.cwd;
        }
        if version.is_none() {
            version = record.version;
        }

        match record_type {
            "user" => {
                user_count += 1;
                if first_user_message.is_none() {
                    if let Some(ref msg) = record.message {
                        if let Some(ref content) = msg.content {
                            let text = extract_text_content(content);
                            if !text.is_empty() {
                                let truncated = truncate_str(&text, 200);
                                first_user_message = Some(truncated);
                            }
                        }
                    }
                }
            }
            "assistant" => {
                if let Some(ref msg) = record.message {
                    // Track model
                    if let Some(ref model) = msg.model {
                        models_set.insert(model.clone());
                    }

                    // Extract tool calls
                    if let Some(ref content) = msg.content {
                        let tools = extract_tool_calls(content);
                        if let Some(ref req_id) = record.request_id {
                            let entry = seen_tool_calls.entry(req_id.clone()).or_default();
                            for tool in &tools {
                                *entry.entry(tool.tool_name.clone()).or_insert(0) += 1;
                            }
                        } else {
                            // No requestId, count directly
                            for tool in &tools {
                                *tool_counts.entry(tool.tool_name.clone()).or_insert(0) += 1;
                            }
                        }
                    }

                    // Track usage by requestId (keep last)
                    let usage = msg.usage.as_ref().map(|u| RawUsage {
                        input_tokens: u.input_tokens,
                        output_tokens: u.output_tokens,
                        cache_read_input_tokens: u.cache_read_input_tokens,
                        cache_creation_input_tokens: u.cache_creation_input_tokens,
                    });

                    if let Some(ref req_id) = record.request_id {
                        assistant_request_ids.insert(req_id.clone(), usage);
                    } else {
                        // No requestId, count usage directly
                        if let Some(u) = usage {
                            input_tokens += u.input_tokens.unwrap_or(0);
                            output_tokens += u.output_tokens.unwrap_or(0);
                            cache_read_tokens += u.cache_read_input_tokens.unwrap_or(0);
                            cache_creation_tokens += u.cache_creation_input_tokens.unwrap_or(0);
                        }
                        assistant_count += 1;
                    }
                }
            }
            _ => {} // tool_result already handled by timestamps above
        }
    }

    // Aggregate deduplicated assistant data
    assistant_count += assistant_request_ids.len() as u64;
    for (_req_id, usage) in &assistant_request_ids {
        if let Some(u) = usage {
            input_tokens += u.input_tokens.unwrap_or(0);
            output_tokens += u.output_tokens.unwrap_or(0);
            cache_read_tokens += u.cache_read_input_tokens.unwrap_or(0);
            cache_creation_tokens += u.cache_creation_input_tokens.unwrap_or(0);
        }
    }

    // Merge tool counts from deduplicated assistant lines
    // For tool calls, we want the total unique tool calls across all streaming lines
    // per requestId. Since tool_use blocks accumulate, take the final count per requestId.
    for (_req_id, counts) in &seen_tool_calls {
        for (tool, count) in counts {
            *tool_counts.entry(tool.clone()).or_insert(0) += count;
        }
    }

    // Calculate duration
    let duration_ms = match (&first_timestamp, &last_timestamp) {
        (Some(first), Some(last)) => {
            let first_dt = chrono::DateTime::parse_from_rfc3339(first).ok();
            let last_dt = chrono::DateTime::parse_from_rfc3339(last).ok();
            match (first_dt, last_dt) {
                (Some(f), Some(l)) => (l - f).num_milliseconds().max(0) as u64,
                _ => 0,
            }
        }
        _ => 0,
    };

    let mut models_used: Vec<String> = models_set.into_iter().collect();
    models_used.sort();

    Ok(SessionSummary {
        session_id: session_id.to_string(),
        first_timestamp,
        last_timestamp,
        duration_ms,
        user_message_count: user_count,
        assistant_message_count: assistant_count,
        total_input_tokens: input_tokens,
        total_output_tokens: output_tokens,
        total_cache_read_tokens: cache_read_tokens,
        total_cache_creation_tokens: cache_creation_tokens,
        models_used,
        git_branch,
        cwd,
        tool_counts,
        first_user_message,
        version,
    })
}

/// Parse a single JSONL file into a full transcript for the detail view.
/// Merges streaming assistant lines by requestId (keeps last per requestId).
pub fn parse_session_detail(path: &Path, session_id: &str) -> Result<SessionDetail> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    // We'll collect all records, then merge assistant lines by requestId.
    // Use a two-pass approach: first collect, then build messages.

    struct PendingAssistant {
        uuid: Option<String>,
        timestamp: Option<String>,
        model: Option<String>,
        content_preview: String,
        tool_calls: Vec<ToolCallInfo>,
        usage: Option<MessageUsage>,
        order: usize, // preserve insertion order
    }

    let mut messages: Vec<(usize, SessionMessage)> = Vec::new();
    let mut assistant_by_request: HashMap<String, PendingAssistant> = HashMap::new();
    let mut order_counter: usize = 0;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.is_empty() {
            continue;
        }

        // Quick-reject
        if line.contains("\"type\":\"file-history-snapshot\"")
            || line.contains("\"type\":\"progress\"")
            || line.contains("\"type\":\"bash_progress\"")
            || line.contains("\"type\":\"summary\"")
        {
            continue;
        }

        let record: RawRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let record_type = record.record_type.as_deref().unwrap_or("");

        match record_type {
            "user" => {
                let content_preview = record
                    .message
                    .as_ref()
                    .and_then(|m| m.content.as_ref())
                    .map(|c| truncate_str(&extract_text_content(c), 500))
                    .unwrap_or_default();

                messages.push((
                    order_counter,
                    SessionMessage {
                        uuid: record.uuid,
                        role: "user".to_string(),
                        timestamp: record.timestamp,
                        model: None,
                        content_preview,
                        tool_calls: Vec::new(),
                        usage: None,
                    },
                ));
                order_counter += 1;
            }
            "assistant" => {
                let msg = record.message.as_ref();

                let content_preview = msg
                    .and_then(|m| m.content.as_ref())
                    .map(|c| truncate_str(&extract_text_content(c), 500))
                    .unwrap_or_default();

                let tool_calls = msg
                    .and_then(|m| m.content.as_ref())
                    .map(|c| extract_tool_calls(c))
                    .unwrap_or_default();

                let usage = msg.and_then(|m| {
                    m.usage.as_ref().map(|u| MessageUsage {
                        input_tokens: u.input_tokens.unwrap_or(0),
                        output_tokens: u.output_tokens.unwrap_or(0),
                        cache_read_input_tokens: u.cache_read_input_tokens.unwrap_or(0),
                        cache_creation_input_tokens: u.cache_creation_input_tokens.unwrap_or(0),
                    })
                });

                let model = msg.and_then(|m| m.model.clone());

                if let Some(ref req_id) = record.request_id {
                    let entry = assistant_by_request
                        .entry(req_id.clone())
                        .or_insert_with(|| {
                            let ord = order_counter;
                            order_counter += 1;
                            PendingAssistant {
                                uuid: record.uuid.clone(),
                                timestamp: record.timestamp.clone(),
                                model: model.clone(),
                                content_preview: String::new(),
                                tool_calls: Vec::new(),
                                usage: None,
                                order: ord,
                            }
                        });

                    // Always update to latest values (streaming accumulation)
                    if !content_preview.is_empty() {
                        entry.content_preview = content_preview;
                    }
                    if !tool_calls.is_empty() {
                        entry.tool_calls = tool_calls;
                    }
                    if usage.is_some() {
                        entry.usage = usage;
                    }
                    if record.timestamp.is_some() {
                        entry.timestamp = record.timestamp;
                    }
                    entry.uuid = record.uuid;
                    if model.is_some() {
                        entry.model = model;
                    }
                } else {
                    messages.push((
                        order_counter,
                        SessionMessage {
                            uuid: record.uuid,
                            role: "assistant".to_string(),
                            timestamp: record.timestamp,
                            model,
                            content_preview,
                            tool_calls,
                            usage,
                        },
                    ));
                    order_counter += 1;
                }
            }
            "tool_result" => {
                // Include tool results in the conversation timeline
                let content_preview = record
                    .message
                    .as_ref()
                    .and_then(|m| m.content.as_ref())
                    .map(|c| truncate_str(&extract_text_content(c), 500))
                    .unwrap_or_else(|| "(tool result)".to_string());

                messages.push((
                    order_counter,
                    SessionMessage {
                        uuid: record.uuid,
                        role: "tool_result".to_string(),
                        timestamp: record.timestamp,
                        model: None,
                        content_preview,
                        tool_calls: Vec::new(),
                        usage: None,
                    },
                ));
                order_counter += 1;
            }
            _ => {}
        }
    }

    // Merge pending assistant messages
    for (_req_id, pending) in assistant_by_request {
        messages.push((
            pending.order,
            SessionMessage {
                uuid: pending.uuid,
                role: "assistant".to_string(),
                timestamp: pending.timestamp,
                model: pending.model,
                content_preview: pending.content_preview,
                tool_calls: pending.tool_calls,
                usage: pending.usage,
            },
        ));
    }

    // Sort by insertion order to maintain conversation flow
    messages.sort_by_key(|(order, _)| *order);

    let final_messages: Vec<SessionMessage> = messages.into_iter().map(|(_, m)| m).collect();

    Ok(SessionDetail {
        session_id: session_id.to_string(),
        messages: final_messages,
    })
}

// ─── Content extraction helpers ─────────────────────────────────────────────

/// Extract text from `content` field which can be a string or an array of content blocks.
pub fn extract_text_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut texts = Vec::new();
            for block in arr {
                if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                    if block_type == "text" {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            texts.push(text.to_string());
                        }
                    }
                }
            }
            texts.join("\n")
        }
        _ => String::new(),
    }
}

/// Extract `tool_use` entries from a content array.
pub fn extract_tool_calls(content: &serde_json::Value) -> Vec<ToolCallInfo> {
    let mut tools = Vec::new();
    if let serde_json::Value::Array(arr) = content {
        for block in arr {
            if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                if block_type == "tool_use" {
                    let tool_name = block
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let tool_id = block
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string());
                    tools.push(ToolCallInfo { tool_name, tool_id });
                }
            }
        }
    }
    tools
}

/// Truncate a string to approximately `max_chars` characters, breaking at a word boundary.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    // Find the byte index of the max_chars-th character
    let byte_end = trimmed
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());
    let truncated = &trimmed[..byte_end];
    // Find last space before the limit for a clean word break
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &trimmed[..last_space])
    } else {
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_project_folder() {
        let decoded = decode_project_folder("C--code-claude-code-tool-manager");
        assert!(
            decoded.contains("code-claude-code-tool-manager"),
            "decoded: {}",
            decoded
        );
        // Should start with drive letter
        assert!(decoded.starts_with("C:"), "decoded: {}", decoded);
    }

    #[test]
    fn test_decode_single_segment() {
        let decoded = decode_project_folder("my-project");
        assert_eq!(decoded, "my-project");
    }

    #[test]
    fn test_extract_text_content_string() {
        let content = serde_json::json!("Hello world");
        assert_eq!(extract_text_content(&content), "Hello world");
    }

    #[test]
    fn test_extract_text_content_array() {
        let content = serde_json::json!([
            {"type": "text", "text": "Hello"},
            {"type": "tool_use", "name": "Read", "id": "123"},
            {"type": "text", "text": "World"}
        ]);
        assert_eq!(extract_text_content(&content), "Hello\nWorld");
    }

    #[test]
    fn test_extract_tool_calls() {
        let content = serde_json::json!([
            {"type": "text", "text": "Let me read that file."},
            {"type": "tool_use", "name": "Read", "id": "toolu_123"},
            {"type": "tool_use", "name": "Glob", "id": "toolu_456"}
        ]);
        let tools = extract_tool_calls(&content);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].tool_name, "Read");
        assert_eq!(tools[0].tool_id, Some("toolu_123".to_string()));
        assert_eq!(tools[1].tool_name, "Glob");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 200), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        let long = "a ".repeat(200);
        let truncated = truncate_str(&long, 20);
        assert!(truncated.len() < 30);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_str_unicode() {
        // Emojis are multi-byte: should not panic when truncation lands mid-character
        let emoji_str = "Hello 🌍🌎🌏 world! 🎉🎊✨ some more text here to exceed the limit";
        let truncated = truncate_str(emoji_str, 10);
        assert!(truncated.ends_with("..."));
        // Ensure it doesn't panic with all-emoji strings
        let all_emoji = "🔥".repeat(50);
        let truncated2 = truncate_str(&all_emoji, 5);
        assert!(truncated2.ends_with("..."));
    }

    #[test]
    fn test_nonexistent_projects_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nonexistent = dir.path().join("nope");
        let result = list_projects_from_dir(&nonexistent).unwrap();
        assert!(!result.exists);
        assert!(result.projects.is_empty());
    }

    #[test]
    fn test_parse_session_summary_basic() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test-session.jsonl");

        let lines = vec![
            r#"{"type":"user","uuid":"u1","sessionId":"test-session","timestamp":"2026-01-15T10:00:00.000Z","version":"2.1.39","gitBranch":"main","cwd":"/code/test","message":{"role":"user","content":"Fix the bug"}}"#,
            r#"{"type":"assistant","uuid":"a1","sessionId":"test-session","timestamp":"2026-01-15T10:00:05.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"I'll fix that for you."},{"type":"tool_use","name":"Read","id":"t1"}],"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}}}"#,
            r#"{"type":"assistant","uuid":"a2","sessionId":"test-session","timestamp":"2026-01-15T10:00:10.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"I'll fix that for you. Done!"},{"type":"tool_use","name":"Read","id":"t1"},{"type":"tool_use","name":"Edit","id":"t2"}],"usage":{"input_tokens":100,"output_tokens":80,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}}}"#,
        ];

        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test-session").unwrap();
        assert_eq!(summary.session_id, "test-session");
        assert_eq!(summary.user_message_count, 1);
        assert_eq!(summary.assistant_message_count, 1); // deduplicated by requestId
        assert_eq!(summary.total_output_tokens, 80); // last usage wins
        assert_eq!(summary.git_branch, Some("main".to_string()));
        assert_eq!(summary.models_used, vec!["claude-opus-4-6".to_string()]);
        assert_eq!(summary.first_user_message, Some("Fix the bug".to_string()));
        assert!(summary.tool_counts.contains_key("Read"));
        assert!(summary.tool_counts.contains_key("Edit"));
    }

    #[test]
    fn test_parse_session_detail_dedup() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("detail-session.jsonl");

        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"Hello"}}"#,
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:02.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"Hi"}],"usage":{"input_tokens":10,"output_tokens":5}}}"#,
            r#"{"type":"assistant","uuid":"a2","timestamp":"2026-01-15T10:00:04.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"Hi there!"}],"usage":{"input_tokens":10,"output_tokens":15}}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:00:10.000Z","message":{"role":"user","content":"Thanks"}}"#,
        ];

        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "detail-session").unwrap();
        assert_eq!(detail.messages.len(), 3); // user, assistant (deduped), user
        assert_eq!(detail.messages[0].role, "user");
        assert_eq!(detail.messages[1].role, "assistant");
        assert_eq!(detail.messages[1].content_preview, "Hi there!"); // last wins
        assert_eq!(detail.messages[1].usage.as_ref().unwrap().output_tokens, 15);
        assert_eq!(detail.messages[2].role, "user");
    }

    #[test]
    fn test_empty_projects_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = list_projects_from_dir(dir.path()).unwrap();
        assert!(result.exists);
        assert!(result.projects.is_empty());
    }

    #[test]
    fn test_list_sessions_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let nonexistent = dir.path().join("nope");
        let result = list_sessions_from_dir(&nonexistent, "nope").unwrap();
        assert!(!result.exists);
        assert!(result.sessions.is_empty());
    }

    // =========================================================================
    // Additional coverage for uncovered paths
    // =========================================================================

    #[test]
    fn test_decode_project_folder_multiple_segments() {
        let decoded = decode_project_folder("Users--tyler--Code--project");
        assert!(decoded.starts_with("Users:"));
        assert!(decoded.contains("tyler"));
        assert!(decoded.contains("Code"));
        assert!(decoded.contains("project"));
    }

    #[test]
    fn test_extract_text_content_non_string_non_array() {
        // Numbers, booleans, etc. should return empty string
        assert_eq!(extract_text_content(&serde_json::json!(42)), "");
        assert_eq!(extract_text_content(&serde_json::json!(true)), "");
        assert_eq!(extract_text_content(&serde_json::json!(null)), "");
    }

    #[test]
    fn test_extract_text_content_array_no_text_blocks() {
        let content = serde_json::json!([
            {"type": "tool_use", "name": "Read", "id": "123"},
            {"type": "image", "source": {}}
        ]);
        assert_eq!(extract_text_content(&content), "");
    }

    #[test]
    fn test_extract_text_content_array_missing_type() {
        let content = serde_json::json!([
            {"no_type_key": "text"},
            {"type": "text", "text": "found"}
        ]);
        assert_eq!(extract_text_content(&content), "found");
    }

    #[test]
    fn test_extract_tool_calls_non_array() {
        let tools = extract_tool_calls(&serde_json::json!("not an array"));
        assert!(tools.is_empty());
    }

    #[test]
    fn test_extract_tool_calls_missing_name() {
        let content = serde_json::json!([
            {"type": "tool_use"}
        ]);
        let tools = extract_tool_calls(&content);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].tool_name, "unknown");
        assert!(tools[0].tool_id.is_none());
    }

    #[test]
    fn test_truncate_str_exact_length() {
        let s = "abcde";
        assert_eq!(truncate_str(s, 5), "abcde");
    }

    #[test]
    fn test_truncate_str_no_space() {
        let s = "abcdefghijklmnop";
        let result = truncate_str(s, 10);
        assert!(result.ends_with("..."));
        assert_eq!(result, "abcdefghij...");
    }

    #[test]
    fn test_truncate_str_whitespace_trimmed() {
        assert_eq!(truncate_str("  hello  ", 200), "hello");
    }

    #[test]
    fn test_parse_session_summary_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("empty.jsonl");
        std::fs::write(&session_file, "").unwrap();

        let summary = parse_session_summary(&session_file, "empty").unwrap();
        assert_eq!(summary.session_id, "empty");
        assert_eq!(summary.user_message_count, 0);
        assert_eq!(summary.assistant_message_count, 0);
        assert_eq!(summary.duration_ms, 0);
    }

    #[test]
    fn test_parse_session_summary_skips_file_history_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"file-history-snapshot","data":{}}"#,
            r#"{"type":"progress","data":{}}"#,
            r#"{"type":"bash_progress","data":{}}"#,
            r#"{"type":"summary","data":{}}"#,
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"test"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.user_message_count, 1);
    }

    #[test]
    fn test_parse_session_summary_skips_invalid_json_lines() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            "not valid json",
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"ok"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.user_message_count, 1);
    }

    #[test]
    fn test_parse_session_summary_tool_result_tracked() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"tool_result","uuid":"t1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"tool","content":"result"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        // tool_result is not user or assistant
        assert_eq!(summary.user_message_count, 0);
        assert_eq!(summary.assistant_message_count, 0);
        // But timestamp should be tracked
        assert!(summary.first_timestamp.is_some());
    }

    #[test]
    fn test_parse_session_summary_assistant_without_request_id() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"hi"}],"usage":{"input_tokens":50,"output_tokens":25}}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.assistant_message_count, 1);
        assert_eq!(summary.total_input_tokens, 50);
        assert_eq!(summary.total_output_tokens, 25);
    }

    #[test]
    fn test_parse_session_summary_assistant_tool_calls_no_request_id() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"assistant","content":[{"type":"tool_use","name":"Bash","id":"t1"}]}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(*summary.tool_counts.get("Bash").unwrap(), 1);
    }

    #[test]
    fn test_parse_session_detail_tool_result_included() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"hi"}}"#,
            r#"{"type":"tool_result","uuid":"t1","timestamp":"2026-01-15T10:00:01.000Z","message":{"role":"tool","content":"result data"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[1].role, "tool_result");
    }

    #[test]
    fn test_parse_session_detail_tool_result_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines =
            vec![r#"{"type":"tool_result","uuid":"t1","timestamp":"2026-01-15T10:00:01.000Z"}"#];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].content_preview, "(tool result)");
    }

    #[test]
    fn test_parse_session_detail_assistant_without_request_id() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"hello"}],"usage":{"input_tokens":10,"output_tokens":5}}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].role, "assistant");
        assert_eq!(detail.messages[0].content_preview, "hello");
    }

    #[test]
    fn test_parse_session_detail_unknown_type_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"unknown_type","uuid":"x1","timestamp":"2026-01-15T10:00:00.000Z"}"#,
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:01.000Z","message":{"role":"user","content":"hi"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].role, "user");
    }

    #[test]
    fn test_list_sessions_skips_non_jsonl_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("readme.txt"), "hello").unwrap();
        std::fs::write(dir.path().join("config.json"), "{}").unwrap();

        let result = list_sessions_from_dir(dir.path(), "test").unwrap();
        assert!(result.exists);
        assert!(result.sessions.is_empty());
    }

    #[test]
    fn test_list_projects_skips_non_dirs() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a-file.txt"), "hello").unwrap();

        let result = list_projects_from_dir(dir.path()).unwrap();
        assert!(result.exists);
        assert!(result.projects.is_empty());
    }

    #[test]
    fn test_list_projects_skips_empty_project_dirs() {
        let dir = tempfile::tempdir().unwrap();
        // Create project dir with no JSONL files
        let proj = dir.path().join("my-project");
        std::fs::create_dir(&proj).unwrap();
        std::fs::write(proj.join("readme.md"), "empty project").unwrap();

        let result = list_projects_from_dir(dir.path()).unwrap();
        assert!(result.projects.is_empty());
    }

    #[test]
    fn test_list_projects_aggregates_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let proj = dir.path().join("project-a");
        std::fs::create_dir(&proj).unwrap();

        let line = r#"{"type":"user","uuid":"u1","sessionId":"s1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"hello"}}"#;
        std::fs::write(proj.join("s1.jsonl"), line).unwrap();

        let result = list_projects_from_dir(dir.path()).unwrap();
        assert_eq!(result.projects.len(), 1);
        assert_eq!(result.projects[0].session_count, 1);
    }

    #[test]
    fn test_session_summary_duration_calculation() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"start"}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:05:00.000Z","message":{"role":"user","content":"end"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.duration_ms, 300000); // 5 minutes
    }

    #[test]
    fn test_session_summary_invalid_timestamps() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"not-a-date","message":{"role":"user","content":"hi"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.duration_ms, 0);
    }

    #[test]
    fn test_projects_dir_returns_path() {
        let dir = projects_dir();
        assert!(dir.to_string_lossy().contains(".claude"));
        assert!(dir.to_string_lossy().contains("projects"));
    }

    #[test]
    fn test_parse_session_summary_earliest_latest_tracking() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T12:00:00.000Z","message":{"role":"user","content":"mid"}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"early"}}"#,
            r#"{"type":"user","uuid":"u3","timestamp":"2026-01-15T14:00:00.000Z","message":{"role":"user","content":"late"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(
            summary.first_timestamp,
            Some("2026-01-15T10:00:00.000Z".to_string())
        );
        assert_eq!(
            summary.last_timestamp,
            Some("2026-01-15T14:00:00.000Z".to_string())
        );
    }

    // =========================================================================
    // Additional coverage: parse_session_summary with multiple request IDs
    // =========================================================================

    #[test]
    fn test_parse_session_summary_multiple_request_ids() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"q1"}}"#,
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:01.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"answer 1"}],"usage":{"input_tokens":100,"output_tokens":50}}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:01:00.000Z","message":{"role":"user","content":"q2"}}"#,
            r#"{"type":"assistant","uuid":"a2","timestamp":"2026-01-15T10:01:01.000Z","requestId":"req_002","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"answer 2"}],"usage":{"input_tokens":200,"output_tokens":100}}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.user_message_count, 2);
        assert_eq!(summary.assistant_message_count, 2);
        assert_eq!(summary.total_input_tokens, 300);
        assert_eq!(summary.total_output_tokens, 150);
    }

    #[test]
    fn test_parse_session_summary_no_usage_in_assistant() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"hi"}]}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.assistant_message_count, 1);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
    }

    #[test]
    fn test_parse_session_summary_user_message_content_array() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":[{"type":"text","text":"Hello from array content"}]}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(
            summary.first_user_message,
            Some("Hello from array content".to_string())
        );
    }

    #[test]
    fn test_parse_session_summary_first_user_message_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":""}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:00:01.000Z","message":{"role":"user","content":"Second message"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        // Empty first message should be skipped, second should be captured
        assert_eq!(
            summary.first_user_message,
            Some("Second message".to_string())
        );
    }

    #[test]
    fn test_parse_session_summary_cwd_and_version() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","version":"2.3.0","cwd":"/home/user/project","message":{"role":"user","content":"hi"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        assert_eq!(summary.version, Some("2.3.0".to_string()));
        assert_eq!(summary.cwd, Some("/home/user/project".to_string()));
    }

    // =========================================================================
    // Additional coverage: parse_session_detail streaming with tool calls
    // =========================================================================

    #[test]
    fn test_parse_session_detail_streaming_updates_tool_calls() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"step1"}]}}"#,
            r#"{"type":"assistant","uuid":"a2","timestamp":"2026-01-15T10:00:01.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"text","text":"step1 done"},{"type":"tool_use","name":"Read","id":"t1"}],"usage":{"input_tokens":100,"output_tokens":50}}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].content_preview, "step1 done");
        assert_eq!(detail.messages[0].tool_calls.len(), 1);
        assert_eq!(detail.messages[0].tool_calls[0].tool_name, "Read");
        assert_eq!(detail.messages[0].usage.as_ref().unwrap().output_tokens, 50);
    }

    #[test]
    fn test_parse_session_detail_preserves_order() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"first"}}"#,
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:01.000Z","requestId":"req_001","message":{"role":"assistant","content":[{"type":"text","text":"response"}]}}"#,
            r#"{"type":"tool_result","uuid":"t1","timestamp":"2026-01-15T10:00:02.000Z","message":{"role":"tool","content":"result"}}"#,
            r#"{"type":"user","uuid":"u2","timestamp":"2026-01-15T10:00:03.000Z","message":{"role":"user","content":"followup"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 4);
        assert_eq!(detail.messages[0].role, "user");
        assert_eq!(detail.messages[1].role, "assistant");
        assert_eq!(detail.messages[2].role, "tool_result");
        assert_eq!(detail.messages[3].role, "user");
    }

    // =========================================================================
    // Additional coverage: list_projects_from_dir multiple projects sorting
    // =========================================================================

    #[test]
    fn test_list_projects_sorted_by_latest_session() {
        let dir = tempfile::tempdir().unwrap();

        // Project A: older session
        let proj_a = dir.path().join("project-a");
        std::fs::create_dir(&proj_a).unwrap();
        std::fs::write(
            proj_a.join("s1.jsonl"),
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-10T10:00:00.000Z","message":{"role":"user","content":"old"}}"#,
        ).unwrap();

        // Project B: newer session
        let proj_b = dir.path().join("project-b");
        std::fs::create_dir(&proj_b).unwrap();
        std::fs::write(
            proj_b.join("s1.jsonl"),
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-20T10:00:00.000Z","message":{"role":"user","content":"new"}}"#,
        ).unwrap();

        let result = list_projects_from_dir(dir.path()).unwrap();
        assert_eq!(result.projects.len(), 2);
        // Project B should be first (more recent)
        assert_eq!(result.projects[0].folder_name, "project-b");
        assert_eq!(result.projects[1].folder_name, "project-a");
    }

    // =========================================================================
    // Additional coverage: decode_project_folder edge cases
    // =========================================================================

    #[test]
    fn test_decode_project_folder_empty() {
        let decoded = decode_project_folder("");
        assert_eq!(decoded, "");
    }

    #[test]
    fn test_decode_project_folder_no_separator() {
        let decoded = decode_project_folder("simple-name");
        assert_eq!(decoded, "simple-name");
    }

    // =========================================================================
    // Additional coverage: extract_text_content empty array
    // =========================================================================

    #[test]
    fn test_extract_text_content_empty_array() {
        let content = serde_json::json!([]);
        assert_eq!(extract_text_content(&content), "");
    }

    // =========================================================================
    // Additional coverage: list_sessions sorts by first_timestamp desc
    // =========================================================================

    #[test]
    fn test_list_sessions_sorted_by_first_timestamp() {
        let dir = tempfile::tempdir().unwrap();

        // Session A: earlier
        std::fs::write(
            dir.path().join("session-a.jsonl"),
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-10T10:00:00.000Z","message":{"role":"user","content":"a"}}"#,
        ).unwrap();

        // Session B: later
        std::fs::write(
            dir.path().join("session-b.jsonl"),
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-20T10:00:00.000Z","message":{"role":"user","content":"b"}}"#,
        ).unwrap();

        let result = list_sessions_from_dir(dir.path(), "test").unwrap();
        assert_eq!(result.sessions.len(), 2);
        // Session B should be first (more recent)
        assert_eq!(result.sessions[0].session_id, "session-b");
        assert_eq!(result.sessions[1].session_id, "session-a");
    }

    // =========================================================================
    // Additional coverage: parse_session_detail empty file
    // =========================================================================

    #[test]
    fn test_parse_session_detail_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("empty.jsonl");
        std::fs::write(&session_file, "").unwrap();

        let detail = parse_session_detail(&session_file, "empty").unwrap();
        assert_eq!(detail.session_id, "empty");
        assert!(detail.messages.is_empty());
    }

    #[test]
    fn test_parse_session_detail_skips_progress_types() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"file-history-snapshot","data":{}}"#,
            r#"{"type":"progress","data":{}}"#,
            r#"{"type":"bash_progress","data":{}}"#,
            r#"{"type":"summary","data":{}}"#,
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"hello"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].role, "user");
    }

    #[test]
    fn test_parse_session_detail_invalid_json_lines_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            "not json",
            "",
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"ok"}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let detail = parse_session_detail(&session_file, "test").unwrap();
        assert_eq!(detail.messages.len(), 1);
    }

    // =========================================================================
    // Additional coverage: aggregate tool_counts from multiple request IDs
    // =========================================================================

    #[test]
    fn test_parse_session_summary_merges_tool_counts() {
        let dir = tempfile::tempdir().unwrap();
        let session_file = dir.path().join("test.jsonl");
        let lines = vec![
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:00.000Z","requestId":"req_001","message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","id":"t1"},{"type":"tool_use","name":"Read","id":"t2"}]}}"#,
            r#"{"type":"assistant","uuid":"a2","timestamp":"2026-01-15T10:01:00.000Z","requestId":"req_002","message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","id":"t3"},{"type":"tool_use","name":"Edit","id":"t4"}]}}"#,
        ];
        std::fs::write(&session_file, lines.join("\n")).unwrap();

        let summary = parse_session_summary(&session_file, "test").unwrap();
        // Read: 2 from req_001 + 1 from req_002 = 3
        assert_eq!(*summary.tool_counts.get("Read").unwrap(), 3);
        assert_eq!(*summary.tool_counts.get("Edit").unwrap(), 1);
    }

    // =========================================================================
    // Struct serialization coverage
    // =========================================================================

    #[test]
    fn test_session_message_serialization() {
        let msg = SessionMessage {
            uuid: Some("uuid-123".to_string()),
            role: "user".to_string(),
            timestamp: Some("2026-01-15T10:00:00Z".to_string()),
            model: None,
            content_preview: "Hello".to_string(),
            tool_calls: vec![],
            usage: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"contentPreview\":\"Hello\""));
    }

    #[test]
    fn test_message_usage_serialization() {
        let usage = MessageUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: 10,
            cache_creation_input_tokens: 5,
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"inputTokens\":100"));
        assert!(json.contains("\"outputTokens\":50"));
        assert!(json.contains("\"cacheReadInputTokens\":10"));
        assert!(json.contains("\"cacheCreationInputTokens\":5"));
    }

    #[test]
    fn test_tool_call_info_serialization() {
        let tool = ToolCallInfo {
            tool_name: "Read".to_string(),
            tool_id: Some("toolu_abc".to_string()),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"toolName\":\"Read\""));
        assert!(json.contains("\"toolId\":\"toolu_abc\""));
    }

    // =========================================================================
    // Additional struct serialization/deserialization round-trip tests
    // =========================================================================

    #[test]
    fn test_project_list_info_serialization() {
        let info = ProjectListInfo {
            dir_path: "/home/user/.claude/projects".to_string(),
            exists: true,
            projects: vec![],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"dirPath\""));
        assert!(json.contains("\"exists\":true"));

        let parsed: ProjectListInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.dir_path, info.dir_path);
        assert_eq!(parsed.exists, info.exists);
    }

    #[test]
    fn test_project_summary_serialization_roundtrip() {
        let mut tool_usage = HashMap::new();
        tool_usage.insert("Read".to_string(), 42);
        tool_usage.insert("Write".to_string(), 10);

        let summary = ProjectSummary {
            folder_name: "my-project".to_string(),
            inferred_path: "/home/user/my-project".to_string(),
            session_count: 5,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_cache_read_tokens: 200,
            total_cache_creation_tokens: 50,
            models_used: vec![
                "claude-opus-4-6".to_string(),
                "claude-sonnet-4-20250514".to_string(),
            ],
            tool_usage,
            earliest_session: Some("2026-01-01T00:00:00Z".to_string()),
            latest_session: Some("2026-03-15T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let parsed: ProjectSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.folder_name, "my-project");
        assert_eq!(parsed.session_count, 5);
        assert_eq!(parsed.total_input_tokens, 1000);
        assert_eq!(parsed.models_used.len(), 2);
        assert_eq!(*parsed.tool_usage.get("Read").unwrap(), 42);
    }

    #[test]
    fn test_session_list_info_serialization_roundtrip() {
        let info = SessionListInfo {
            project_folder: "my-project".to_string(),
            exists: true,
            sessions: vec![],
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: SessionListInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.project_folder, "my-project");
        assert!(parsed.exists);
        assert!(parsed.sessions.is_empty());
    }

    #[test]
    fn test_session_summary_serialization_roundtrip() {
        let mut tool_counts = HashMap::new();
        tool_counts.insert("Read".to_string(), 5);

        let summary = SessionSummary {
            session_id: "abc-123".to_string(),
            first_timestamp: Some("2026-01-15T10:00:00Z".to_string()),
            last_timestamp: Some("2026-01-15T11:00:00Z".to_string()),
            duration_ms: 3600000,
            user_message_count: 10,
            assistant_message_count: 10,
            total_input_tokens: 5000,
            total_output_tokens: 2000,
            total_cache_read_tokens: 1000,
            total_cache_creation_tokens: 100,
            models_used: vec!["claude-opus-4-6".to_string()],
            git_branch: Some("main".to_string()),
            cwd: Some("/home/user/project".to_string()),
            tool_counts,
            first_user_message: Some("Fix the bug".to_string()),
            version: Some("2.3.0".to_string()),
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"sessionId\":\"abc-123\""));
        assert!(json.contains("\"gitBranch\":\"main\""));
        assert!(json.contains("\"firstUserMessage\":\"Fix the bug\""));

        let parsed: SessionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, "abc-123");
        assert_eq!(parsed.duration_ms, 3600000);
        assert_eq!(parsed.version, Some("2.3.0".to_string()));
    }

    #[test]
    fn test_session_detail_serialization_roundtrip() {
        let detail = SessionDetail {
            session_id: "detail-1".to_string(),
            messages: vec![
                SessionMessage {
                    uuid: Some("u1".to_string()),
                    role: "user".to_string(),
                    timestamp: Some("2026-01-15T10:00:00Z".to_string()),
                    model: None,
                    content_preview: "Hello".to_string(),
                    tool_calls: vec![],
                    usage: None,
                },
                SessionMessage {
                    uuid: Some("a1".to_string()),
                    role: "assistant".to_string(),
                    timestamp: Some("2026-01-15T10:00:01Z".to_string()),
                    model: Some("claude-opus-4-6".to_string()),
                    content_preview: "Hi there!".to_string(),
                    tool_calls: vec![ToolCallInfo {
                        tool_name: "Read".to_string(),
                        tool_id: Some("t1".to_string()),
                    }],
                    usage: Some(MessageUsage {
                        input_tokens: 100,
                        output_tokens: 50,
                        cache_read_input_tokens: 10,
                        cache_creation_input_tokens: 5,
                    }),
                },
            ],
        };

        let json = serde_json::to_string(&detail).unwrap();
        let parsed: SessionDetail = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.session_id, "detail-1");
        assert_eq!(parsed.messages.len(), 2);
        assert_eq!(parsed.messages[1].tool_calls.len(), 1);
        assert_eq!(parsed.messages[1].usage.as_ref().unwrap().input_tokens, 100);
    }

    // =========================================================================
    // Additional coverage: list_projects_from_dir with tool usage aggregation
    // =========================================================================

    #[test]
    fn test_list_projects_aggregates_tool_usage() {
        let dir = tempfile::tempdir().unwrap();
        let proj = dir.path().join("project-tools");
        std::fs::create_dir(&proj).unwrap();

        // Session 1: uses Read and Write
        let lines1 = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"q1"}}"#,
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-15T10:00:01.000Z","requestId":"req_001","message":{"role":"assistant","model":"claude-opus-4-6","content":[{"type":"tool_use","name":"Read","id":"t1"},{"type":"tool_use","name":"Write","id":"t2"}],"usage":{"input_tokens":100,"output_tokens":50}}}"#,
        ];
        std::fs::write(proj.join("s1.jsonl"), lines1.join("\n")).unwrap();

        // Session 2: uses Read and Bash
        let lines2 = vec![
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-20T10:00:00.000Z","message":{"role":"user","content":"q2"}}"#,
            r#"{"type":"assistant","uuid":"a1","timestamp":"2026-01-20T10:00:01.000Z","requestId":"req_002","message":{"role":"assistant","model":"claude-sonnet-4-20250514","content":[{"type":"tool_use","name":"Read","id":"t3"},{"type":"tool_use","name":"Bash","id":"t4"}],"usage":{"input_tokens":200,"output_tokens":100}}}"#,
        ];
        std::fs::write(proj.join("s2.jsonl"), lines2.join("\n")).unwrap();

        let result = list_projects_from_dir(dir.path()).unwrap();
        assert_eq!(result.projects.len(), 1);

        let proj_summary = &result.projects[0];
        assert_eq!(proj_summary.session_count, 2);
        assert_eq!(proj_summary.total_input_tokens, 300);
        assert_eq!(proj_summary.total_output_tokens, 150);

        // Tool usage should be aggregated
        assert_eq!(*proj_summary.tool_usage.get("Read").unwrap(), 2);
        assert_eq!(*proj_summary.tool_usage.get("Write").unwrap(), 1);
        assert_eq!(*proj_summary.tool_usage.get("Bash").unwrap(), 1);

        // Models should be sorted
        assert!(proj_summary
            .models_used
            .contains(&"claude-opus-4-6".to_string()));
        assert!(proj_summary
            .models_used
            .contains(&"claude-sonnet-4-20250514".to_string()));

        // Earliest/latest
        assert_eq!(
            proj_summary.earliest_session,
            Some("2026-01-15T10:00:00.000Z".to_string())
        );
        assert_eq!(
            proj_summary.latest_session,
            Some("2026-01-20T10:00:01.000Z".to_string())
        );
    }

    // =========================================================================
    // Additional coverage: message_usage deserialization
    // =========================================================================

    #[test]
    fn test_message_usage_deserialization() {
        let json = r#"{"inputTokens":100,"outputTokens":50,"cacheReadInputTokens":10,"cacheCreationInputTokens":5}"#;
        let usage: MessageUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_read_input_tokens, 10);
        assert_eq!(usage.cache_creation_input_tokens, 5);
    }

    #[test]
    fn test_tool_call_info_deserialization() {
        let json = r#"{"toolName":"Bash","toolId":"toolu_xyz"}"#;
        let tool: ToolCallInfo = serde_json::from_str(json).unwrap();
        assert_eq!(tool.tool_name, "Bash");
        assert_eq!(tool.tool_id, Some("toolu_xyz".to_string()));
    }

    #[test]
    fn test_tool_call_info_no_id() {
        let tool = ToolCallInfo {
            tool_name: "Read".to_string(),
            tool_id: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"toolName\":\"Read\""));
        assert!(json.contains("\"toolId\":null"));
    }

    // =========================================================================
    // Additional coverage: session_message deserialization
    // =========================================================================

    #[test]
    fn test_session_message_deserialization() {
        let json = r#"{"uuid":"u1","role":"user","timestamp":"2026-01-15T10:00:00Z","model":null,"contentPreview":"Hello","toolCalls":[],"usage":null}"#;
        let msg: SessionMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.uuid, Some("u1".to_string()));
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content_preview, "Hello");
        assert!(msg.model.is_none());
    }

    // =========================================================================
    // Additional coverage: list_sessions_from_dir with unparseable session
    // =========================================================================

    #[test]
    fn test_list_sessions_skips_unparseable_sessions() {
        let dir = tempfile::tempdir().unwrap();

        // Valid session
        std::fs::write(
            dir.path().join("good.jsonl"),
            r#"{"type":"user","uuid":"u1","timestamp":"2026-01-15T10:00:00.000Z","message":{"role":"user","content":"hi"}}"#,
        ).unwrap();

        // Invalid session (only invalid json lines)
        std::fs::write(
            dir.path().join("bad.jsonl"),
            "not json at all\nalso not json\n",
        )
        .unwrap();

        let result = list_sessions_from_dir(dir.path(), "test").unwrap();
        // Both files are .jsonl so both get attempted, but bad one should still parse
        // (it just has 0 messages). The function won't error on empty sessions.
        assert!(result.sessions.len() >= 1);
    }

    // =========================================================================
    // Additional coverage: extract_text_content with text blocks missing text field
    // =========================================================================

    #[test]
    fn test_extract_text_content_text_block_missing_text_field() {
        let content = serde_json::json!([
            {"type": "text"},  // missing "text" key
            {"type": "text", "text": "found"}
        ]);
        assert_eq!(extract_text_content(&content), "found");
    }
}
