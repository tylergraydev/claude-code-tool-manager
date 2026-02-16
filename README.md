# Claude Code Tool Manager

[![Build Status](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml/badge.svg)](https://github.com/tylergraydev/claude-code-tool-manager/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/tylergraydev/claude-code-tool-manager)](https://github.com/tylergraydev/claude-code-tool-manager/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/tylergraydev/claude-code-tool-manager/total)](https://github.com/tylergraydev/claude-code-tool-manager/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A desktop app for managing MCP servers, Commands, Skills, Sub-Agents, and Hooks across multiple AI coding assistants.

### Supported Editors

| Editor | Config Location | Format |
|--------|-----------------|--------|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code) | `~/.claude.json` | JSON |
| [OpenCode](https://opencode.ai) | `~/.config/opencode/opencode.json` | JSON |
| [Codex CLI](https://github.com/openai/codex) | `~/.codex/config.toml` | TOML |
| [GitHub Copilot CLI](https://githubnext.com/projects/copilot-cli) | `~/.copilot/mcp-config.json` | JSON |
| [Cursor](https://cursor.com) | `~/.cursor/mcp.json` | JSON |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli) | `~/.gemini/settings.json` | JSON |

Enable the editors you use in Settings, and your MCP configurations will sync to all of them automatically.

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

**Profiles** — Save and restore named configurations of your globally-enabled MCPs, skills, subagents, commands, and hooks. Snapshot your current setup, then switch between "Work" and "Personal" configurations with one click.

**Status Line Builder** — Design custom terminal status lines for Claude Code with a visual builder. Choose from 25+ segment types (model, cost, context window, git branch, vim mode, token counts, usage stats, and more), apply Powerline themes, or browse and install premade status lines from the community gallery.

**Spinner Verbs** — Customize the action verbs shown in the Claude Code spinner while it works (e.g., "Pondering", "Crafting", "Scheming"). Add, edit, delete, and reorder verbs with drag-and-drop, toggle individual verbs on/off, and choose between append or replace mode. Syncs directly to `~/.claude/settings.json`.

**Projects** — Scan for existing projects, assign tools globally or per-project, and keep everything in sync with Claude's config files. Search and filter to quickly find projects and available tools.

## Installation

### macOS (Homebrew)

```bash
brew tap tylergraydev/cctm
brew install --cask claude-code-tool-manager
```

### Direct Download

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

## Multi-Editor Sync

When you enable multiple editors in Settings:

- **Global MCPs** sync to each editor's global config file
- **Project MCPs** sync to project-level configs (e.g., `.cursor/mcp.json`, `.gemini/settings.json`)
- **Detection** finds installed editors via PATH or app bundles

This means you can manage your MCP configurations once and have them available in Claude Code, Cursor, Gemini CLI, and more simultaneously.

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
