# Claude Code Tool Manager

A desktop application for managing MCP servers, Skills, and Sub-Agents for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

![Claude Code Tool Manager Screenshot](imgs/Screenshot%202025-12-19%20132108.png)

## Why?

Claude Code stores its configuration across multiple JSON files and markdown files scattered throughout your filesystem. Managing MCPs, Skills, and Sub-Agents manually means editing `~/.claude.json`, creating files in `~/.claude/commands/`, and keeping track of what's enabled where.

**Claude Code Tool Manager** gives you a visual interface to:
- See all your MCPs, Skills, and Sub-Agents in one place
- Quickly enable/disable tools per project or globally
- Import MCP configs by pasting JSON or CLI commands
- Auto-detect existing configurations from your filesystem

## Features

### MCP Server Management
- **MCP Library**: Create, edit, and organize MCP server configurations
- **Project-based assignments**: Assign specific MCPs to individual projects
- **Global settings**: Enable MCPs globally across all projects
- **Paste-to-import**: Quickly import MCP configs from JSON or `claude mcp add` commands
- **Multiple transport types**: Support for stdio, HTTP, and SSE MCP servers

### Skills (Slash Commands)
- **Command Skills**: Create slash commands (`/name`) that users can invoke
- **Agent Skills**: Define skills that the model can invoke automatically
- **Tool restrictions**: Limit which tools a skill can use
- **Argument hints**: Provide usage hints for command arguments
- **Global/project assignment**: Enable skills globally or per-project
- **Auto-detection**: Automatically discovers skills from `~/.claude/commands/` and project `.claude/commands/` directories

### Sub-Agents
- **Custom sub-agents**: Define specialized agents for specific tasks
- **Model selection**: Choose which Claude model the sub-agent uses
- **Tool access control**: Specify which tools the sub-agent can access
- **Auto-detection**: Automatically discovers agents from `~/.claude/agents/` and project `.claude/agents/` directories
- **Project scoping**: Assign sub-agents globally or to specific projects

### Project Management
- **Project scanning**: Automatically detect projects from existing Claude configs
- **Project details**: View and manage MCPs, Skills, and Sub-Agents per project
- **Configuration sync**: Changes are automatically written to Claude config files

### Additional Features
- **Dark mode**: Toggle between light and dark themes
- **Search & filter**: Quickly find MCPs, Skills, and Sub-Agents
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
4. **Create Projects**: Go to "Projects" and add your project directories
5. **Assign Tools**: Open a project and assign MCPs, Skills, or Sub-Agents

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

### MCP Types

#### stdio (Standard I/O)
Local command-line tools like npm packages:
```json
{
  "command": "npx",
  "args": ["-y", "@package/mcp-server"],
  "env": { "API_KEY": "xxx" }
}
```

#### SSE (Server-Sent Events)
Cloud services with SSE endpoints:
```json
{
  "type": "sse",
  "url": "https://mcp.service.com/sse"
}
```

#### HTTP (REST API)
REST APIs with token authentication:
```json
{
  "type": "http",
  "url": "https://api.service.com/mcp",
  "headers": { "Authorization": "Bearer ${TOKEN}" }
}
```

### Configuration Files

Claude Code Tool Manager reads from and writes to the standard Claude configuration locations:

| Type | Location |
|------|----------|
| Global MCPs | `~/.claude.json` |
| Global Skills | `~/.claude/commands/` |
| Global Sub-Agents | `~/.claude/agents/` |
| Project MCPs | `{project}/.claude/.mcp.json` |
| Project Skills | `{project}/.claude/commands/` |
| Project Sub-Agents | `{project}/.claude/agents/` |

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

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
