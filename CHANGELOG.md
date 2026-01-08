# Changelog

All notable changes to Claude Code Tool Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.8.1] - 2026-01-07

### Fixed
- Version numbers now properly embedded in v1.8.0 release artifacts
- Auto-updater will correctly detect version 1.8.1 as newer than 1.7.0

## [1.8.0] - 2026-01-07

### Added
- **Lazy-Loading MCP Gateway**: New approach to MCP tool management that reduces context pollution
  - Gateway provides 3 meta-tools instead of exposing all backend tools upfront
  - `list_available_mcps`: Discover available MCP servers
  - `load_mcp_tools`: Connect to an MCP and get its tools
  - `call_mcp_tool`: Execute a tool on a specific MCP
- **Connection Management**: Backend manager with lazy connection initialization
  - MCPs loaded from database but not connected until requested
  - Namespaced tool names (e.g., `mcp_name__tool_name`)
  - Connection status tracking (Connecting, Connected, Disconnected, Failed, Restarting)
  - Backend restart capability
- **Transport Layer**: Uses rmcp's StreamableHttpService for HTTP transport
- **Server Features**: Axum server with CORS support and graceful shutdown handling

### Fixed
- Vitest clipboard mock now uses `Object.defineProperty` for happy-dom compatibility
- ComponentExports test paths now use `$lib` alias
- Added `$app/stores` mock for Sidebar.svelte imports
- Skipped flaky ComponentExports tests in CI environment

## [1.5.0] - 2025-12-23

### Added
- **Built-in MCP Server**: Expose Tool Manager functionality as an MCP server
  - 31 tools for programmatic management of MCPs, Skills, Sub-Agents, Hooks, and Projects
  - Streamable HTTP transport on configurable port (default: 23847)
  - Enable/disable and configure from Settings page
  - Automatically adds itself to MCP library when enabled
  - Tools include: `list_mcps`, `create_mcp`, `update_mcp`, `delete_mcp`, `assign_mcp_to_project`, `list_skills`, `create_skill`, `list_subagents`, `create_subagent`, `list_hooks`, `create_hook`, and more
- **MCP Gateway Server**: Aggregate multiple MCP servers into a single endpoint
  - Combines tools from multiple backend MCP servers
  - Tool names prefixed with source MCP (e.g., `filesystem__read_file`)
  - Streamable HTTP transport on configurable port (default: 23848)
  - Add/remove backend MCPs dynamically from Settings
- **Settings UI Enhancements**: New sections for managing built-in servers
  - Start/stop controls for both MCP Server and Gateway
  - Port configuration with validation
  - Connection config display for easy setup
  - Backend MCP management for Gateway

### Fixed
- Built-in MCP Server now properly exposes tools via `tools/list` (added missing `#[tool_handler]` macro)

### Changed
- Extended `mcp_client` service with Streamable HTTP protocol support
- Added comprehensive unit tests for all database operations
- Refactored Tauri commands to use testable helper functions

## [1.4.0] - 2025-12-22

### Added
- **MCP Execution & Exploration**: Interactively execute MCP tools directly from the Tool Manager
  - **Session Management**: Persistent sessions for stdio MCP servers with proper lifecycle management
  - **Tool Execution**: Execute any MCP tool with dynamic form-based parameter input
  - **JSON Schema Form**: Dynamic form generation from JSON schema for tool parameters
  - **Result Visualization**: Display tool results with JSON formatting and copy functionality
  - **Execution History**: Track and re-run previous tool executions within a session
- **E2E Test Infrastructure**: Playwright test framework setup for end-to-end testing
- **Rust Test CI/CD**: Separate GitHub Actions workflow for Rust tests, Clippy, and formatting checks

### Changed
- Improved Rust test coverage from 46% to 64%
- Applied consistent code formatting across all Rust files (cargo fmt)

### Fixed
- Debug logger tests marked as flaky (require serial execution)
- CI workflow now builds frontend before running Rust tests

## [1.3.10] - 2025-12-22

### Added
- Debug mode persistence: Debug logging state now persists between app restarts
- Version automation in CI/CD

## [1.3.9] - 2025-12-22

### Fixed
- Default MCP type to "stdio" when type is not specified during import

## [1.3.8] - 2025-12-22

### Added
- Full SSE MCP tool listing support with async implementation

## [1.3.7] - 2025-12-21

### Added
- **HTTP MCP Testing**: Full support for Streamable HTTP transport
  - Session tracking via `mcp-session-id` header
  - Parses both JSON and SSE-formatted responses
  - Lists all available tools with descriptions and schemas
- **SSE MCP Testing**: Basic connectivity verification for SSE transport
  - Verifies SSE endpoint responds correctly
  - Shows "connected" status when successful
  - Note: Full tool listing requires async implementation (coming soon)

### Fixed
- HTTP MCPs now maintain session across initialize/tools requests
- Proper Accept header (`application/json, text/event-stream`) for MCP spec compliance

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

[Unreleased]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.5.0...HEAD
[1.5.0]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.10...v1.4.0
[1.3.10]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.9...v1.3.10
[1.3.9]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.8...v1.3.9
[1.3.8]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.7...v1.3.8
[1.3.7]: https://github.com/tylergraydev/claude-code-tool-manager/compare/v1.3.6...v1.3.7
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
