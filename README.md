# Claude Code Tool Manager

[![Build Status](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml/badge.svg)](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/tylergraydev/claude-code-tool-manager)](https://github.com/tylergraydev/claude-code-tool-manager/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/tylergraydev/claude-code-tool-manager/total)](https://github.com/tylergraydev/claude-code-tool-manager/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A desktop app for managing MCP servers, Commands, Skills, Sub-Agents, and Hooks for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) and [OpenCode](https://opencode.ai).

![Claude Code Tool Manager](imgs/project-assign.gif)

## The Problem

Claude Code configuration is scattered across your filesystem—`~/.claude.json`, `~/.claude/commands/`, project-level configs—and managing it means hand-editing JSON and markdown files.

**Claude Code Tool Manager** gives you a single visual interface to see everything, quickly toggle tools on/off per project, and import configs by pasting JSON or CLI commands.

## Features

**MCP Testing** — Connect to any MCP server and execute its tools directly from the app. Validate your configurations actually work before assigning them to projects.

![MCP Testing](imgs/mcp-testing-12s.gif)

**AI-Controllable** — Ships with its own MCP server, so Claude Code (or any MCP client) can manage your tool configurations programmatically. Ask Claude to "add this MCP to my project" and it just works.

**MCP Servers** — Create, edit, and organize MCP configurations. Supports stdio, HTTP, and SSE transports. Import by pasting JSON or `claude mcp add` commands.

**Commands** — Create and manage slash commands (invoked via `/command-name`). Stored as single markdown files in `.claude/commands/`.

**Skills** — Build agent skills that Claude can invoke automatically based on context. Stored as directories with `SKILL.md` and supporting files in `.claude/skills/`.

**Sub-Agents** — Define custom sub-agents with specialized capabilities. Auto-discovers from `~/.claude/agents/`.

**Hooks** — Create and manage Claude Code hooks that run on events like task completion, notifications, and tool use. Includes a sound notification wizard to play sounds when Claude needs your attention or finishes a task.

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

| Scope | MCPs | Commands | Skills | Sub-Agents | Hooks |
|-------|------|----------|--------|------------|-------|
| Global | `~/.claude.json` | `~/.claude/commands/` | `~/.claude/skills/` | `~/.claude/agents/` | `~/.claude/settings.json` |
| Project | `.claude/.mcp.json` | `.claude/commands/` | `.claude/skills/` | `.claude/agents/` | `.claude/settings.json` |

## What's Next

This tool is expanding beyond Claude Code. Coming soon:

- **OpenAI Codex CLI** — Manage Codex configurations alongside Claude
- **GitHub Copilot CLI** — Unified management across GitHub's AI tools

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
