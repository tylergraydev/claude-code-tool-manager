# Claude Code Tool Manager

[![Build Status](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml/badge.svg)](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/tylergraydev/claude-code-tool-manager)](https://github.com/tylergraydev/claude-code-tool-manager/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/tylergraydev/claude-code-tool-manager/total)](https://github.com/tylergraydev/claude-code-tool-manager/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A desktop app for managing MCP servers, Skills, and Sub-Agents for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

![Claude Code Tool Manager](imgs/project-assign.gif)

## The Problem

Claude Code configuration is scattered across your filesystem—`~/.claude.json`, `~/.claude/commands/`, project-level configs—and managing it means hand-editing JSON and markdown files.

**Claude Code Tool Manager** gives you a single visual interface to see everything, quickly toggle tools on/off per project, and import configs by pasting JSON or CLI commands.

## Features

**MCP Testing** — Connect to any MCP server and execute its tools directly from the app. Validate your configurations actually work before assigning them to projects.

![MCP Testing](imgs/mcp-testing-12s.gif)

**AI-Controllable** — Ships with its own MCP server, so Claude Code (or any MCP client) can manage your tool configurations programmatically. Ask Claude to "add this MCP to my project" and it just works.

**MCP Servers** — Create, edit, and organize MCP configurations. Supports stdio, HTTP, and SSE transports. Import by pasting JSON or `claude mcp add` commands.

**Skills & Sub-Agents** — Manage slash commands, agent skills, and custom sub-agents. Auto-discovers from `~/.claude/commands/` and `~/.claude/agents/`.

**Projects** — Scan for existing projects, assign tools globally or per-project, and keep everything in sync with Claude's config files.

## Installation

Download from [Releases](https://github.com/tylergraydev/claude-code-tool-manager/releases):

| Platform | Download |
|----------|----------|
| Windows | `.msi` or `.exe` |
| macOS | `.dmg` (Intel & Apple Silicon) |
| Linux | `.AppImage` or `.deb` |

<details>
<summary><strong>Build from source</strong></summary>

Requires [Node.js](https://nodejs.org/) 18+, [Rust](https://www.rust-lang.org/tools/install) 1.70+, and [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```bash
git clone https://github.com/tylergraydev/claude-code-tool-manager.git
cd claude-code-tool-manager
npm install
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

</details>

## Quick Start

1. **Scan** — Click "Scan for MCPs" to detect existing configurations
2. **Add** — Go to MCP Library → Add MCP (or paste a config)
3. **Assign** — Open a project and toggle the tools you need

### Importing MCPs

Paste JSON:
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

Or paste a CLI command:
```
claude mcp add filesystem -- npx -y @modelcontextprotocol/server-filesystem ~/Documents
```

## Configuration Locations

| Scope | MCPs | Skills | Sub-Agents |
|-------|------|--------|------------|
| Global | `~/.claude.json` | `~/.claude/commands/` | `~/.claude/agents/` |
| Project | `.claude/.mcp.json` | `.claude/commands/` | `.claude/agents/` |

## What's Next

This tool is expanding beyond Claude Code. Coming soon:

- **OpenAI Codex CLI** — Manage Codex configurations alongside Claude
- **GitHub Copilot CLI** — Unified management across GitHub's AI tools  
- **OpenCode** — Support for the open-source AI coding assistant

The goal: one app to manage all your AI CLI tools, with synchronized configurations across them.

## Development

```bash
npm install          # Install dependencies
npm run tauri dev    # Development mode
npm test             # Run tests
npm run tauri build  # Production build
```

## Tech Stack

Svelte 5 + SvelteKit • Tauri 2 + Rust • SQLite • Tailwind CSS v4

## Contributing

PRs welcome! Fork → branch → commit → PR.

## License

MIT
