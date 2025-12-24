# Claude Code Tool Manager

[![Build Status](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml/badge.svg)](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/tylergraydev/claude-code-tool-manager)](https://github.com/tylergraydev/claude-code-tool-manager/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/tylergraydev/claude-code-tool-manager/total)](https://github.com/tylergraydev/claude-code-tool-manager/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A desktop application for managing MCP servers, Skills, Hooks, and Sub-Agents for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

![Dashboard](imgs/dashboard.png)

## Why?

Claude Code stores its configuration across multiple JSON files and markdown files scattered throughout your filesystem. Managing MCPs, Skills, Hooks, and Sub-Agents manually means editing `~/.claude.json`, `~/.claude/settings.json`, creating files in `~/.claude/commands/`, `~/.claude/skills/`, and keeping track of what's enabled where.

**Claude Code Tool Manager** gives you a visual interface to:
- See all your MCPs, Skills, Hooks, and Sub-Agents in one place
- Quickly enable/disable tools per project or globally
- Import MCP configs by pasting JSON or CLI commands
- Auto-detect existing configurations from your filesystem
- Manage hook event handlers for tool calls

## Features

### MCP Server Management

Create, organize, and assign MCP servers to projects or global settings.

- **MCP Library**: Create, edit, and organize MCP server configurations
- **MCP Testing**: Test MCP servers to verify they work
  - Click the menu button on any MCP card and select "Test"
  - **Stdio MCPs**: Full testing with tool listing (npx, uvx, node commands)
  - **HTTP MCPs**: Full testing with Streamable HTTP transport support
  - **SSE MCPs**: Connectivity verification (full tool listing coming soon)
  - Automatically runs the MCP protocol handshake
  - Lists all available tools with descriptions and input schemas
  - Shows server info, version, and capabilities
  - Helpful error messages for common issues (npm auth, PATH problems)
- **Project-based assignments**: Assign specific MCPs to individual projects
- **Global settings**: Enable MCPs globally across all projects
- **Paste-to-import**: Quickly import MCP configs from JSON or `claude mcp add` commands
- **Multiple transport types**: Support for stdio, HTTP, and SSE MCP servers

![MCP Tool List](imgs/toollist.png)

### MCP Execution & Exploration

Interactively execute MCP tools directly from the Tool Manager to understand what they do and see real results.

- **Tool Discovery**: After a successful MCP test, click "Execute Tools" to browse all available tools
- **Interactive Execution**: Fill in parameters using a dynamic form and execute any tool
- **JSON Schema Forms**: Automatic form generation from tool input schemas with support for all JSON types
- **Result Visualization**: View tool results with JSON formatting and one-click copy
- **Execution History**: Track previous executions within each session and quickly re-run them
- **Session Management**: Persistent connections to stdio MCP servers during your session

![Execute MCP Tool](imgs/executemcptool.png)

![Execution History](imgs/executionhistory.png)

### Built-in MCP Server

Expose Tool Manager as an MCP server that Claude Code can connect to for programmatic management.

- **31 management tools**: Create, update, delete, and manage MCPs, Skills, Sub-Agents, Hooks, and Projects
- **Streamable HTTP transport**: Modern MCP protocol support
- **Configurable port**: Default 23847, change in Settings
- **Auto-library integration**: Automatically adds connection config to your MCP library when enabled
- **Available tools include**:
  - `list_mcps`, `create_mcp`, `update_mcp`, `delete_mcp`
  - `assign_mcp_to_project`, `remove_mcp_from_project`
  - `list_skills`, `create_skill`, `delete_skill`
  - `list_subagents`, `create_subagent`, `delete_subagent`
  - `list_hooks`, `create_hook`, `delete_hook`
  - `list_projects`, `get_project`
  - Global enable/disable for all tool types

Enable from **Settings > Built-in MCP Server**.

### MCP Gateway

Aggregate multiple MCP servers into a single unified endpoint.

- **Tool aggregation**: Combine tools from multiple backend MCPs into one connection
- **Namespaced tools**: Tool names prefixed with source MCP (e.g., `filesystem__read_file`)
- **Dynamic backends**: Add/remove backend MCPs from Settings without restarting
- **Streamable HTTP transport**: Configurable port (default: 23848)
- **Use case**: Reduce the number of MCP connections Claude Code needs to manage

Enable from **Settings > MCP Gateway**.

### Skills (Slash Commands & Agent Skills)

Define custom slash commands and agent skills with full control over tool access and model behavior.

![Skills](imgs/skills.png)

- **Command Skills**: Create slash commands (`/name`) that users can invoke
- **Agent Skills**: Define skills that the model can invoke automatically (stored in `.claude/skills/`)
- **Tool restrictions**: Limit which tools a skill can use with `allowed-tools`
- **Model override**: Run skills with a specific model (opus, sonnet, haiku)
- **Disable model invocation**: Prevent the model from automatically invoking agent skills
- **Skill files**: Attach reference files, assets, and scripts to agent skills
  - `references/` - Documentation and context files
  - `assets/` - Images, JSON data, templates
  - `scripts/` - Shell scripts for automation
- **Argument hints**: Provide usage hints for command arguments
- **Global/project assignment**: Enable skills globally or per-project
- **Auto-detection**: Discovers skills from `~/.claude/commands/`, `~/.claude/skills/`, and project directories

![Add Skill](imgs/add-skill.png)

### Hooks (Event Handlers)

Configure hooks that run before/after tool calls, on permission requests, and more.

![Hooks](imgs/hooks.png)

- **All event types supported**:
  - `PreToolUse` / `PostToolUse` - Before/after tool execution
  - `PermissionRequest` - When tool needs permission
  - `Notification` - On notifications
  - `UserPromptSubmit` - When user submits prompt
  - `SessionStart` / `SessionEnd` - Session lifecycle
  - `Stop` / `SubagentStop` - On stop events
  - `PreCompact` - Before context compaction
- **Matcher patterns**: Target specific tools with regex patterns (e.g., `Bash`, `Write|Edit`)
- **Command hooks**: Run shell commands on events
- **Prompt hooks**: Inject prompts on events
- **Templates**: Quick-start templates for common hooks (auto-format, protect files, logging)
- **Global/project assignment**: Enable hooks globally or per-project
- **Auto-detection**: Discovers hooks from `~/.claude/settings.json` and project settings

### Sub-Agents

Define specialized agents with custom instructions, tool access, and model selection.

![Sub-Agents](imgs/sub-agent.png)

- **Custom sub-agents**: Define specialized agents for specific tasks
- **Model selection**: Choose which Claude model the sub-agent uses
- **Tool access control**: Specify which tools the sub-agent can access
- **Permission modes**: Configure permission handling (default, acceptEdits, bypassPermissions)
- **Skills integration**: Assign skills to sub-agents
- **Auto-detection**: Discovers agents from `~/.claude/agents/` and project directories
- **Project scoping**: Assign sub-agents globally or to specific projects

![Edit Sub-Agent](imgs/edit-sub-agent.png)

### Project Management

Manage MCPs, Skills, Hooks, and Sub-Agents on a per-project basis.

- **Project scanning**: Automatically detect projects from existing Claude configs
- **Project details**: View and manage all tools per project
- **Configuration sync**: Changes are automatically written to Claude config files
- **Open folder**: Quick access to open project folders in your file manager

### Global Settings

Configure tools that apply across all Claude Code sessions.

- **Global MCPs**: Enable MCP servers for all projects
- **Global Skills**: Make skills available everywhere
- **Global Hooks**: Apply hooks to all sessions
- **Global Sub-Agents**: Use agents in any project

### Marketplace

Browse and import community-created skills and sub-agents.

- **Browse community repos**: Discover Skills and Sub-Agents from curated GitHub repositories
- **One-click import**: Import skills and agents directly into your library
- **Auto-sync on startup**: Repositories are synced automatically when the app launches
- **Add custom repos**: Add your own GitHub repositories to browse
- **File & README parsing**: Supports both file-based repos and README-based awesome lists

### Claude.json Viewer

View and manage the raw Claude Code configuration.

- **Raw config view**: See the actual `~/.claude.json` configuration
- **Per-project configs**: View project-specific MCP configurations
- **Toggle MCPs**: Enable/disable MCPs directly from the viewer

### Additional Features

- **Dark mode**: Toggle between light and dark themes
- **Search & filter**: Quickly find MCPs, Skills, Hooks, and Sub-Agents
- **Auto-sync**: Changes are automatically written to Claude config files

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/tylergraydev/claude-code-tool-manager/releases) page:

- **Windows**: `.msi` installer or `.exe`
- **macOS**: `.dmg` (Intel and Apple Silicon)
- **Linux**: `.AppImage` or `.deb`

### Build from Source

#### Prerequisites
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Platform-specific dependencies for [Tauri](https://tauri.app/start/prerequisites/)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/tylergraydev/claude-code-tool-manager.git
cd claude-code-tool-manager

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Usage

### Quick Start

1. Launch Claude Code Tool Manager
2. **Scan**: Click "Scan for MCPs" to auto-detect existing MCP configurations
3. **Add MCPs**: Go to "MCP Library" and click "Add MCP" to create new configurations
4. **Create Skills**: Go to "Skills" to create slash commands or agent skills
5. **Configure Hooks**: Go to "Hooks" to set up event handlers
6. **Create Projects**: Go to "Projects" and add your project directories
7. **Assign Tools**: Open a project and assign MCPs, Skills, Hooks, or Sub-Agents

### Importing MCP Configurations

You can paste MCP configurations directly:

**JSON format:**
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "~/Documents"]
    }
  }
}
```

**CLI command:**
```bash
claude mcp add filesystem -- npx -y @modelcontextprotocol/server-filesystem ~/Documents
```

### Creating Skills

#### Command Skills (Slash Commands)
Create a slash command that users invoke with `/name`:

```yaml
---
description: Format code with Prettier
allowed-tools: Bash, Read, Write
argument-hint: [file path]
---

Format the specified file using Prettier. If no file is specified, format the current file.
```

#### Agent Skills
Create skills the model can invoke automatically (stored in `.claude/skills/name/SKILL.md`):

```yaml
---
name: code-reviewer
description: Review code for best practices
allowed-tools: Read, Grep, Glob
model: haiku
disable-model-invocation: false
---

Review the code for:
- Security vulnerabilities
- Performance issues
- Best practices
```

Agent skills can include additional files in subdirectories:
- `references/` - Context documents the skill can reference
- `assets/` - Data files, templates, images
- `scripts/` - Helper shell scripts

### Configuration Files

Claude Code Tool Manager reads from and writes to the standard Claude configuration locations:

| Type | Location |
|------|----------|
| Global MCPs | `~/.claude.json` |
| Global Settings/Hooks | `~/.claude/settings.json` |
| Global Command Skills | `~/.claude/commands/*.md` |
| Global Agent Skills | `~/.claude/skills/*/SKILL.md` |
| Global Sub-Agents | `~/.claude/agents/*.md` |
| Project MCPs | `{project}/.claude/.mcp.json` |
| Project Settings/Hooks | `{project}/.claude/settings.local.json` |
| Project Command Skills | `{project}/.claude/commands/*.md` |
| Project Agent Skills | `{project}/.claude/skills/*/SKILL.md` |
| Project Sub-Agents | `{project}/.claude/agents/*.md` |

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/), [SvelteKit](https://kit.svelte.dev/), [Tailwind CSS v4](https://tailwindcss.com/)
- **Backend**: [Tauri 2](https://tauri.app/), [Rust](https://www.rust-lang.org/)
- **Database**: SQLite (via [rusqlite](https://github.com/rusqlite/rusqlite))
- **Icons**: [Lucide](https://lucide.dev/)
- **Testing**: [Vitest](https://vitest.dev/)

## Development

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Run tests
npm test

# Run tests with UI
npm run test:ui

# Type checking
npm run check

# Build for production
npm run tauri build
```

### Project Structure

```
claude-code-tool-manager/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   │   ├── hooks/      # Hook management UI
│   │   │   ├── mcp/        # MCP management UI
│   │   │   ├── skills/     # Skills management UI
│   │   │   └── subagents/  # Sub-agents management UI
│   │   ├── stores/         # Svelte 5 reactive stores
│   │   ├── types/          # TypeScript types
│   │   └── utils/          # Utility functions
│   ├── routes/             # SvelteKit routes
│   └── tests/              # Vitest tests
├── src-tauri/              # Tauri Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # Database schema and models
│       └── services/       # Business logic services
└── package.json
```

## License

MIT

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
