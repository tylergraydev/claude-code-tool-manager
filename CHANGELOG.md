# Changelog

All notable changes to Claude Code Tool Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.6] - 2025-12-21

### Added
- **MCP Testing**: Test stdio MCP servers directly from the app
  - Click "Test" in the MCP card menu to verify a server works
  - Auto-runs MCP protocol handshake and lists available tools
  - Displays server info, tool count, and capabilities
  - Shows tool names, descriptions, and input schemas
  - Helpful error messages for common issues (npm auth, PATH problems)
- **OpenCode Support**: Scan and import projects from OpenCode configurations
  - Auto-detects OpenCode config at `~/.opencode/config.json`
  - Imports MCP server configurations from OpenCode projects

### Changed
- HTTP and SSE MCP testing is disabled for now (coming soon)
- Improved process spawning to properly inherit PATH environment

### Fixed
- npx commands now work correctly by running through shell
- Better error messages for npm authentication issues

## [1.3.5] - 2025-12-21

### Added
- **Debug Mode**: Enable file-based logging from Global Settings to help troubleshoot issues
  - Toggle in Settings > Global Settings
  - Captures Rust backend logs, frontend console logs, and Tauri invoke calls
  - Log files saved to app data directory
  - "Open Folder" button for easy access to log files
- Debug logging throughout the application for better diagnostics
- Screenshots added to README documentation

### Fixed
- "Ambiguous column name: created_at" SQL error when loading hooks
  - Fixed JOIN queries in hooks commands to use table-prefixed column names

### Changed
- Bug report template now includes instructions for attaching debug logs

## [1.3.4] - 2025-12-21

### Added
- Marketplace sort options: sort MCPs by "Recently Updated" (default) or "Name (A-Z)"
- `updatedAt` field to track when MCPs were last updated in the registry

### Changed
- Cleaned up debug logging from previous releases

## [1.3.3] - 2025-12-20

### Fixed
- MCP Registry loading with dynamic JSON parsing
- Duplicate MCPs in marketplace by deduplicating by registryId

## [1.3.2] - 2025-12-20

### Fixed
- MCP Registry API response parsing
- JSON deserialization for MCP Registry

## [1.3.1] - 2025-12-20

### Added
- What's New modal that displays after auto-updates

## [1.3.0] - 2025-12-20

### Added
- **MCP Registry Integration**: Browse and import MCPs from the official [MCP Registry](https://registry.modelcontextprotocol.io/)
  - Official Registry API integration
  - Search MCPs by name or description
  - Paginated listing with "Load More" support
  - One-click import with environment variable placeholders
  - Multi-package support (npm via `npx -y`, PyPI via `uvx`, Docker)
  - Environment variable display with descriptions

### Fixed
- Build workflow signing for CI/CD

## [1.2.0] - 2025-12-20

### Added
- Auto-updater for seamless updates

## [1.1.0] - 2025-12-20

### Added
- **Marketplace**: Browse and import Skills and Sub-Agents from GitHub repositories
  - Browse community repos for Skills and Sub-Agents
  - One-click import to library
  - Auto-sync on startup
  - Add custom GitHub repositories
  - Support for file-based repos and README-based repos (awesome lists)
- Default repositories: [wshobson/commands](https://github.com/wshobson/commands), [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code)

### Changed
- Version now displays dynamically from app config

### Fixed
- Import status resets when items are deleted from library

## [1.0.1] - 2025-12-20

### Fixed
- Global MCP config now correctly writes to `~/.claude.json` instead of `~/.claude/settings.json`
- Global Settings page displays correct config file path
- Backup functionality now includes `claude.json`

## [1.0.0] - 2025-12-20

### Added
- Initial release
- **MCP Server Management**
  - MCP Library for creating, editing, and organizing configurations
  - Project-based MCP assignments
  - Global MCP settings
  - Paste-to-import for JSON and CLI commands
  - Support for stdio, HTTP, and SSE transport types
- **Skills (Slash Commands)**
  - Command Skills for user-invocable slash commands
  - Agent Skills for model-invocable actions
  - Auto-detection from `~/.claude/commands/` and project directories
  - Global and per-project assignment
- **Sub-Agents**
  - Custom sub-agent definitions
  - Model selection per sub-agent
  - Auto-detection from `~/.claude/agents/` and project directories
  - Project scoping
- **Project Management**
  - Automatic project scanning
  - Per-project tool management
  - Configuration sync to Claude config files
- **Additional Features**
  - Dark mode
  - Search and filter

[Unreleased]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.6...HEAD
[1.3.6]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.5...v1.3.6
[1.3.5]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.4...v1.3.5
[1.3.4]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.3...v1.3.4
[1.3.3]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.2...v1.3.3
[1.3.2]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/tylergraydev/claude-code-tool-manager/releases/tag/v1.0.0
