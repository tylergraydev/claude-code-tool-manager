use regex::Regex;
use serde::{Deserialize, Serialize};

/// Files that should be skipped when parsing repos
const JUNK_FILES: &[&str] = &[
    "readme.md",
    "readme",
    "contributing.md",
    "contributing",
    "license.md",
    "license",
    "changelog.md",
    "changelog",
    "code_of_conduct.md",
    "security.md",
    "pull_request_template.md",
    "issue_template.md",
    "authors.md",
    "contributors.md",
    "history.md",
    "todo.md",
    "roadmap.md",
    "acknowledgments.md",
    "acknowledgements.md",
    "claude.md",
];

/// Directories that should be skipped
const JUNK_DIRS: &[&str] = &[
    ".github",
    ".vscode",
    ".idea",
    "node_modules",
    "dist",
    "build",
    "docs",
    "__pycache__",
    ".git",
];

/// Check if a file path should be skipped
pub fn should_skip_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    let file_name = path_lower.split('/').last().unwrap_or(&path_lower);

    // Check if it's a junk file
    if JUNK_FILES.iter().any(|junk| file_name == *junk) {
        return true;
    }

    // Check if it's in a junk directory
    if JUNK_DIRS.iter().any(|dir| {
        path_lower.contains(&format!("/{}/", dir)) || path_lower.starts_with(&format!("{}/", dir))
    }) {
        return true;
    }

    // Skip files that start with a dot (hidden files)
    if file_name.starts_with('.') {
        return true;
    }

    // Skip files that are clearly not content (templates, examples docs)
    if file_name.contains("template")
        || file_name.contains("example") && !path_lower.contains("commands")
    {
        return true;
    }

    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedItem {
    pub name: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub raw_content: Option<String>,
    pub file_path: Option<String>,
    pub item_type: String, // "mcp", "skill", "subagent"
    pub metadata: Option<String>,
}

/// Check if a URL is valid for an MCP entry (not an image or badge)
fn is_valid_mcp_url(url: &str) -> bool {
    let lower = url.to_lowercase();

    // Must be an http/https URL
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return false;
    }

    // Must not be an image file
    if lower.ends_with(".svg")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".gif")
    {
        return false;
    }

    // Must not be a badge service
    if lower.contains("shields.io") || lower.contains("img.shields") || lower.contains("badge") {
        return false;
    }

    true
}

/// Parse a README file for MCP entries (awesome-list format)
/// Looks for patterns like:
/// - [Name](url) - Description
/// - | Name | Description | Link |
pub fn parse_readme_for_mcps(content: &str) -> Vec<ParsedItem> {
    let mut items = Vec::new();

    // Pattern 1: Markdown links with descriptions (list items)
    // - [Name](url) - Description
    let link_pattern = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)\s*[-–:]\s*(.+)").unwrap();

    for cap in link_pattern.captures_iter(content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let url = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(3).map(|m| m.as_str().trim().to_string());

        if is_valid_mcp_url(&url) && !items.iter().any(|i: &ParsedItem| i.name == name) {
            items.push(ParsedItem {
                name,
                description,
                source_url: Some(url),
                raw_content: None,
                file_path: None,
                item_type: "mcp".to_string(),
                metadata: None,
            });
        }
    }

    // Pattern 2: Markdown tables
    // | Name | Description | ... |
    let table_pattern = Regex::new(r"\|\s*\[([^\]]+)\]\(([^)]+)\)\s*\|([^|]*)\|").unwrap();

    for cap in table_pattern.captures_iter(content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let url = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(3).map(|m| m.as_str().trim().to_string());

        if is_valid_mcp_url(&url) && !items.iter().any(|i: &ParsedItem| i.name == name) {
            items.push(ParsedItem {
                name,
                description,
                source_url: Some(url),
                raw_content: None,
                file_path: None,
                item_type: "mcp".to_string(),
                metadata: None,
            });
        }
    }

    items
}

/// Check if a name looks like a valid skill/command name
fn is_valid_skill_name(name: &str) -> bool {
    // Must be at least 2 characters
    if name.len() < 2 {
        return false;
    }

    // Must not be just numbers or dashes
    if name.chars().all(|c| c.is_numeric() || c == '-') {
        return false;
    }

    // Must not be an image file
    let lower = name.to_lowercase();
    if lower.ends_with(".svg")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".gif")
    {
        return false;
    }

    true
}

/// Check if a URL looks like a valid skill file (not an image)
fn is_valid_skill_url(url: &str) -> bool {
    let lower = url.to_lowercase();

    // Must not be an image file
    if lower.ends_with(".svg")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".gif")
    {
        return false;
    }

    // Must not be a shields.io badge URL
    if lower.contains("shields.io") || lower.contains("img.shields") {
        return false;
    }

    true
}

/// Parse a README file for skill entries
/// Looks for skill/command references in various formats
pub fn parse_readme_for_skills(content: &str) -> Vec<ParsedItem> {
    let mut items = Vec::new();

    // Pattern 1: HTML anchor with image badge (awesome-claude-code format)
    // <a href="https://github.com/.../commands/name.md"><img ... alt="/name"></a>
    // followed by _description_ on the next line
    let html_pattern =
        Regex::new(r#"<a\s+href="([^"]+\.md)"[^>]*>.*?</a>\s*(?:<br\s*/?>)?\s*_([^_]+)_"#).unwrap();

    for cap in html_pattern.captures_iter(content) {
        let url = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(2).map(|m| m.as_str().trim().to_string());

        // Extract name from URL path (e.g., /commands/commit.md -> commit)
        let name = url
            .split('/')
            .last()
            .unwrap_or("")
            .trim_end_matches(".md")
            .to_string();

        if is_valid_skill_name(&name)
            && is_valid_skill_url(&url)
            && !items.iter().any(|i: &ParsedItem| i.name == name)
        {
            items.push(ParsedItem {
                name,
                description,
                source_url: Some(url),
                raw_content: None,
                file_path: None,
                item_type: "skill".to_string(),
                metadata: None,
            });
        }
    }

    // Pattern 2: /command-name - Description (must be at start of line or after whitespace)
    let command_pattern =
        Regex::new(r"(?m)^\s*[-*]?\s*/([a-zA-Z][a-zA-Z0-9_-]+)\s*[-–:]\s*(.+)").unwrap();

    for cap in command_pattern.captures_iter(content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(2).map(|m| m.as_str().trim().to_string());

        if is_valid_skill_name(&name) && !items.iter().any(|i: &ParsedItem| i.name == name) {
            items.push(ParsedItem {
                name,
                description,
                source_url: None,
                raw_content: None,
                file_path: None,
                item_type: "skill".to_string(),
                metadata: None,
            });
        }
    }

    // Pattern 3: Markdown links to .md files in list format
    // - [Name](file.md) or - [Name](file.md) - Description
    let md_link_pattern =
        Regex::new(r"(?m)^\s*[-*]\s*\[([^\]]+)\]\(([^)]+\.md)\)(?:\s*[-–:]?\s*(.*))?").unwrap();

    for cap in md_link_pattern.captures_iter(content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let url = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(3).map(|m| m.as_str().trim().to_string());

        // Validate name and URL
        if !is_valid_skill_name(&name) || !is_valid_skill_url(&url) {
            continue;
        }

        if !items.iter().any(|i: &ParsedItem| i.name == name) {
            items.push(ParsedItem {
                name,
                description: if description.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
                    None
                } else {
                    description
                },
                source_url: Some(url),
                raw_content: None,
                file_path: None,
                item_type: "skill".to_string(),
                metadata: None,
            });
        }
    }

    // Pattern 4: GitHub repo links with descriptions (common awesome-list format)
    // - [repo-name](https://github.com/user/repo) - Description
    let github_link_pattern =
        Regex::new(r"(?m)^\s*[-*]\s*\[([^\]]+)\]\((https://github\.com/[^)]+)\)\s*[-–:]\s*(.+)")
            .unwrap();

    for cap in github_link_pattern.captures_iter(content) {
        let name = cap
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let url = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let description = cap.get(3).map(|m| m.as_str().trim().to_string());

        if is_valid_skill_name(&name) && !items.iter().any(|i: &ParsedItem| i.name == name) {
            items.push(ParsedItem {
                name,
                description,
                source_url: Some(url),
                raw_content: None,
                file_path: None,
                item_type: "skill".to_string(),
                metadata: None,
            });
        }
    }

    items
}

/// Parse a markdown skill/command file
/// Extracts frontmatter and content
pub fn parse_skill_file(content: &str, file_path: &str) -> Option<ParsedItem> {
    let file_name = file_path.split('/').last().unwrap_or(file_path);
    let name = file_name.trim_end_matches(".md");

    // Parse YAML frontmatter if present
    let (frontmatter, body) = parse_frontmatter(content);

    let description = frontmatter
        .get("description")
        .cloned()
        .or_else(|| extract_first_paragraph(body));

    let skill_type =
        if frontmatter.contains_key("allowed-tools") || frontmatter.contains_key("allowedTools") {
            "skill"
        } else {
            "command"
        };

    Some(ParsedItem {
        name: name.to_string(),
        description,
        source_url: None,
        raw_content: Some(content.to_string()),
        file_path: Some(file_path.to_string()),
        item_type: skill_type.to_string(),
        metadata: serde_json::to_string(&frontmatter).ok(),
    })
}

/// Parse a markdown subagent file
pub fn parse_subagent_file(content: &str, file_path: &str) -> Option<ParsedItem> {
    let file_name = file_path.split('/').last().unwrap_or(file_path);
    let name = file_name.trim_end_matches(".md");

    let (frontmatter, body) = parse_frontmatter(content);

    let description = frontmatter
        .get("description")
        .cloned()
        .or_else(|| extract_first_paragraph(body));

    Some(ParsedItem {
        name: name.to_string(),
        description,
        source_url: None,
        raw_content: Some(content.to_string()),
        file_path: Some(file_path.to_string()),
        item_type: "subagent".to_string(),
        metadata: serde_json::to_string(&frontmatter).ok(),
    })
}

/// Detect item type from file path and content
pub fn detect_item_type(path: &str, content: &str) -> String {
    let path_lower = path.to_lowercase();

    // Check path for hints
    if path_lower.contains("agent") || path_lower.contains("subagent") {
        return "subagent".to_string();
    }

    if path_lower.contains("command") || path_lower.contains("skill") {
        return "skill".to_string();
    }

    if path_lower.contains("mcp") || path_lower.contains("server") {
        return "mcp".to_string();
    }

    // Check content for hints
    let (frontmatter, _) = parse_frontmatter(content);

    if frontmatter.contains_key("model") || frontmatter.contains_key("tools") {
        return "subagent".to_string();
    }

    if frontmatter.contains_key("allowed-tools") || frontmatter.contains_key("allowedTools") {
        return "skill".to_string();
    }

    // Default to skill for .md files in commands directory
    "skill".to_string()
}

/// Parse YAML frontmatter from markdown content
pub fn parse_frontmatter(content: &str) -> (std::collections::HashMap<String, String>, &str) {
    let mut frontmatter = std::collections::HashMap::new();

    if !content.starts_with("---") {
        return (frontmatter, content);
    }

    // Find the closing ---
    if let Some(end_pos) = content[3..].find("\n---") {
        let fm_content = &content[3..end_pos + 3];
        let body = &content[end_pos + 7..];

        // Simple YAML parsing (key: value)
        for line in fm_content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase().replace('_', "-");
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !value.is_empty() {
                    frontmatter.insert(key, value);
                }
            }
        }

        return (frontmatter, body);
    }

    (frontmatter, content)
}

/// Extract the first non-empty paragraph from markdown content
fn extract_first_paragraph(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with('-')
            && !line.starts_with('*')
        {
            return Some(line.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // =========================================================================
    // should_skip_file tests
    // =========================================================================

    #[rstest]
    #[case("readme.md", true)]
    #[case("README.md", true)]
    #[case("README", true)]
    #[case("contributing.md", true)]
    #[case("CONTRIBUTING.md", true)]
    #[case("license.md", true)]
    #[case("LICENSE", true)]
    #[case("changelog.md", true)]
    #[case("code_of_conduct.md", true)]
    #[case("security.md", true)]
    #[case("claude.md", true)]
    fn test_should_skip_junk_files(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(should_skip_file(path), expected);
    }

    #[rstest]
    #[case(".github/workflows/ci.yml", true)]
    #[case(".vscode/settings.json", true)]
    #[case("node_modules/package/index.js", true)]
    #[case("dist/bundle.js", true)]
    #[case("build/output.js", true)]
    #[case("__pycache__/module.pyc", true)]
    #[case(".git/config", true)]
    fn test_should_skip_junk_directories(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(should_skip_file(path), expected);
    }

    #[rstest]
    #[case(".hidden-file", true)]
    #[case(".env", true)]
    #[case(".gitignore", true)]
    fn test_should_skip_hidden_files(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(should_skip_file(path), expected);
    }

    #[rstest]
    #[case("commands/review.md", false)]
    #[case("skills/testing/SKILL.md", false)]
    #[case("agents/code-reviewer.md", false)]
    #[case("src/main.rs", false)]
    #[case("tools/mcp-server.json", false)]
    fn test_should_not_skip_valid_files(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(should_skip_file(path), expected);
    }

    // =========================================================================
    // is_valid_mcp_url tests
    // =========================================================================

    #[rstest]
    #[case("https://github.com/owner/repo", true)]
    #[case("http://localhost:3000", true)]
    #[case("https://example.com/mcp-server", true)]
    fn test_is_valid_mcp_url_valid(#[case] url: &str, #[case] expected: bool) {
        assert_eq!(is_valid_mcp_url(url), expected);
    }

    #[rstest]
    #[case("https://shields.io/badge.svg", false)]
    #[case("https://img.shields.io/badge/test.svg", false)]
    #[case("https://example.com/image.png", false)]
    #[case("https://example.com/icon.jpg", false)]
    #[case("https://example.com/logo.gif", false)]
    #[case("relative/path", false)]
    #[case("ftp://example.com", false)]
    #[case("not-a-url", false)]
    fn test_is_valid_mcp_url_invalid(#[case] url: &str, #[case] expected: bool) {
        assert_eq!(is_valid_mcp_url(url), expected);
    }

    // =========================================================================
    // is_valid_skill_name tests
    // =========================================================================

    #[rstest]
    #[case("review", true)]
    #[case("code-review", true)]
    #[case("test_skill", true)]
    #[case("my-awesome-skill", true)]
    fn test_is_valid_skill_name_valid(#[case] name: &str, #[case] expected: bool) {
        assert_eq!(is_valid_skill_name(name), expected);
    }

    #[rstest]
    #[case("a", false)] // Too short
    #[case("---", false)] // Only dashes
    #[case("123", false)] // Only numbers
    #[case("1-2-3", false)] // Only numbers and dashes
    #[case("icon.svg", false)] // Image file
    #[case("logo.png", false)] // Image file
    #[case("photo.jpg", false)] // Image file
    #[case("anim.gif", false)] // Image file
    fn test_is_valid_skill_name_invalid(#[case] name: &str, #[case] expected: bool) {
        assert_eq!(is_valid_skill_name(name), expected);
    }

    // =========================================================================
    // is_valid_skill_url tests
    // =========================================================================

    #[rstest]
    #[case("commands/review.md", true)]
    #[case("https://github.com/user/repo", true)]
    #[case("skills/test.md", true)]
    fn test_is_valid_skill_url_valid(#[case] url: &str, #[case] expected: bool) {
        assert_eq!(is_valid_skill_url(url), expected);
    }

    #[rstest]
    #[case("icon.svg", false)]
    #[case("https://shields.io/badge.svg", false)]
    #[case("image.png", false)]
    fn test_is_valid_skill_url_invalid(#[case] url: &str, #[case] expected: bool) {
        assert_eq!(is_valid_skill_url(url), expected);
    }

    // =========================================================================
    // parse_readme_for_mcps tests
    // =========================================================================

    #[test]
    fn test_parse_readme_for_mcps_list_format() {
        let content = r#"
# Awesome MCP Servers

- [GitHub MCP](https://github.com/github/github-mcp-server) - Official GitHub MCP server
- [Filesystem](https://github.com/modelcontextprotocol/servers) - Local filesystem access
"#;

        let items = parse_readme_for_mcps(content);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "GitHub MCP");
        assert!(items[0].description.as_ref().unwrap().contains("Official"));
    }

    #[test]
    fn test_parse_readme_for_mcps_table_format() {
        let content = r#"
| Name | Description |
|------|-------------|
| [MCP Server](https://github.com/x/y) | A great server |
"#;

        let items = parse_readme_for_mcps(content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "MCP Server");
    }

    #[test]
    fn test_parse_readme_for_mcps_filters_badges() {
        let content = r#"
- [Valid MCP](https://github.com/owner/repo) - A valid MCP
- [Badge](https://shields.io/badge.svg) - Should be filtered
- [Image](https://example.com/logo.png) - Should be filtered
"#;

        let items = parse_readme_for_mcps(content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Valid MCP");
    }

    #[test]
    fn test_parse_readme_for_mcps_no_duplicates() {
        let content = r#"
- [Same MCP](https://github.com/owner/repo1) - First entry
- [Same MCP](https://github.com/owner/repo2) - Duplicate name
"#;

        let items = parse_readme_for_mcps(content);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_readme_for_mcps_empty_content() {
        let content = "";
        let items = parse_readme_for_mcps(content);
        assert_eq!(items.len(), 0);
    }

    // =========================================================================
    // parse_skill_file tests
    // =========================================================================

    #[test]
    fn test_parse_skill_file_with_frontmatter() {
        let content = r#"---
description: Review code for issues
allowed-tools: Read, Grep
---

# Code Review

Review the provided code for potential issues.
"#;

        let item = parse_skill_file(content, "commands/review.md").unwrap();
        assert_eq!(item.name, "review");
        assert_eq!(item.item_type, "skill");
        assert!(item.description.unwrap().contains("Review code"));
    }

    #[test]
    fn test_parse_skill_file_command_type() {
        let content = r#"---
description: A simple command
---

Do something simple.
"#;

        let item = parse_skill_file(content, "commands/simple.md").unwrap();
        assert_eq!(item.item_type, "command");
    }

    #[test]
    fn test_parse_skill_file_no_frontmatter() {
        let content = "Just plain content here.";

        let item = parse_skill_file(content, "commands/plain.md").unwrap();
        assert_eq!(item.name, "plain");
        assert_eq!(
            item.description,
            Some("Just plain content here.".to_string())
        );
    }

    #[test]
    fn test_parse_skill_file_nested_path() {
        let content = "---\ndescription: Test\n---\nBody";

        let item = parse_skill_file(content, "some/nested/path/skill-name.md").unwrap();
        assert_eq!(item.name, "skill-name");
    }

    // =========================================================================
    // parse_subagent_file tests
    // =========================================================================

    #[test]
    fn test_parse_subagent_file() {
        let content = r#"---
description: Code review expert
model: claude-3-opus
tools: Read, Grep, Glob
---

You are a code review expert.
"#;

        let item = parse_subagent_file(content, "agents/code-reviewer.md").unwrap();
        assert_eq!(item.name, "code-reviewer");
        assert_eq!(item.item_type, "subagent");
        assert!(item.description.unwrap().contains("Code review"));
    }

    // =========================================================================
    // detect_item_type tests
    // =========================================================================

    #[rstest]
    #[case("agents/helper.md", "", "subagent")]
    #[case("subagents/reviewer.md", "", "subagent")]
    #[case("commands/review.md", "", "skill")]
    #[case("skills/testing.md", "", "skill")]
    #[case("mcp/server.md", "", "mcp")]
    #[case("servers/github.md", "", "mcp")]
    fn test_detect_item_type_from_path(
        #[case] path: &str,
        #[case] content: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(detect_item_type(path, content), expected);
    }

    #[test]
    fn test_detect_item_type_from_content_subagent() {
        let content = "---\nmodel: claude-3-opus\ntools: Read\n---\nBody";
        assert_eq!(detect_item_type("unknown.md", content), "subagent");
    }

    #[test]
    fn test_detect_item_type_from_content_skill() {
        let content = "---\nallowed-tools: Read, Grep\n---\nBody";
        assert_eq!(detect_item_type("unknown.md", content), "skill");
    }

    #[test]
    fn test_detect_item_type_default() {
        assert_eq!(detect_item_type("random.md", "no hints here"), "skill");
    }

    // =========================================================================
    // parse_frontmatter tests
    // =========================================================================

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
description: Test skill
model: claude-3-opus
---
Body content here
"#;

        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.get("description"), Some(&"Test skill".to_string()));
        assert_eq!(fm.get("model"), Some(&"claude-3-opus".to_string()));
        assert!(body.contains("Body content"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "Just body content, no frontmatter";

        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_quoted_values() {
        let content = r#"---
description: "Quoted value"
name: 'Single quoted'
---
Body"#;

        let (fm, _) = parse_frontmatter(content);
        assert_eq!(fm.get("description"), Some(&"Quoted value".to_string()));
        assert_eq!(fm.get("name"), Some(&"Single quoted".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_normalizes_keys() {
        let content = r#"---
allowed_tools: Read
allowedTools: Grep
---
Body"#;

        let (fm, _) = parse_frontmatter(content);
        // Underscores are converted to dashes
        assert!(fm.contains_key("allowed-tools") || fm.contains_key("allowedtools"));
    }

    #[test]
    fn test_parse_frontmatter_empty_values_ignored() {
        let content = r#"---
description: Valid
empty:
---
Body"#;

        let (fm, _) = parse_frontmatter(content);
        assert_eq!(fm.len(), 1);
        assert!(fm.contains_key("description"));
    }

    #[test]
    fn test_parse_frontmatter_unclosed() {
        let content = r#"---
description: Test
no closing delimiter"#;

        let (fm, body) = parse_frontmatter(content);
        // Should return original content when frontmatter is not properly closed
        assert!(fm.is_empty());
        assert_eq!(body, content);
    }

    // =========================================================================
    // parse_readme_for_skills tests
    // =========================================================================

    #[test]
    fn test_parse_readme_for_skills_command_pattern() {
        let content = r#"
## Commands

- /review - Review code for issues
- /commit - Generate commit messages
"#;

        let items = parse_readme_for_skills(content);
        assert!(items.iter().any(|i| i.name == "review"));
        assert!(items.iter().any(|i| i.name == "commit"));
    }

    #[test]
    fn test_parse_readme_for_skills_md_link_pattern() {
        let content = r#"
- [review](commands/review.md) - Review code
- [test](skills/test.md) - Run tests
"#;

        let items = parse_readme_for_skills(content);
        assert!(items.len() >= 2);
    }

    #[test]
    fn test_parse_readme_for_skills_filters_invalid() {
        let content = r#"
- /a - Too short name
- /123 - Only numbers
- [icon.svg](image.svg) - Image file
"#;

        let items = parse_readme_for_skills(content);
        assert_eq!(items.len(), 0);
    }
}
