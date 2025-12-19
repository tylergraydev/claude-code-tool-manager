# Claude MCP Manager

A cross-platform GUI application to manage MCP (Model Context Protocol) servers for Claude Code.

## Features

- **MCP Library**: Central storage of all your MCP configurations
- **Add MCPs**: Easy form-based creation for stdio, SSE, and HTTP types
- **Drag & Drop**: Assign MCPs to projects by dragging from the library
- **Global Settings**: Add MCPs to your global Claude Code settings
- **Toggle On/Off**: Enable/disable MCPs without removing them
- **Auto-detect**: Automatically scans existing MCPs on startup

## Tech Stack

- **Frontend**: Svelte 5 + SvelteKit + Tailwind CSS
- **Backend**: Tauri 2 (Rust)
- **Database**: SQLite

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (latest stable)
- Platform-specific dependencies for Tauri:
  - **Windows**: Microsoft Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

## Development Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Run in development mode:
   ```bash
   npm run tauri dev
   ```

## Building

Create a production build:
```bash
npm run tauri build
```

The installer will be created in `src-tauri/target/release/bundle/`.

## Configuration Paths

The app reads and writes to these Claude Code configuration locations:

| Type | Location |
|------|----------|
| Global plugins | `~/.claude/plugins/marketplaces/{plugin}/.mcp.json` |
| Project MCPs | `./.claude/.mcp.json` (in project root) |
| Global settings | `~/.claude/settings.json` |

## MCP Types

### stdio (Standard I/O)
Local command-line tools like npm packages:
```json
{
  "command": "npx",
  "args": ["-y", "@package/mcp-server"],
  "env": { "API_KEY": "xxx" }
}
```

### SSE (Server-Sent Events)
Cloud services with SSE endpoints:
```json
{
  "type": "sse",
  "url": "https://mcp.service.com/sse"
}
```

### HTTP (REST API)
REST APIs with token authentication:
```json
{
  "type": "http",
  "url": "https://api.service.com/mcp",
  "headers": { "Authorization": "Bearer ${TOKEN}" }
}
```

## License

MIT
