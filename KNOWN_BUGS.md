# Known Bugs

## MCP Servers

### Community MCP servers writing to stdout breaks stdio protocol

**Severity:** High
**Affects:** MCP test modal, MCP execution modal

Some community/marketplace MCP servers print human-readable messages (e.g. `🚀 Hypertool MCP is ready!`) to **stdout** on startup instead of reserving stdout exclusively for JSON-RPC messages. This violates the MCP stdio protocol specification.

**Symptoms:**
- Test modal hangs or shows "process closed stdout unexpectedly"
- MCP client receives non-JSON output and cannot parse the initialize response

**Known affected servers:**
- `@toolprint/hypertool-mcp` — prints `🚀 Hypertool MCP is ready!` to stdout

**Root cause:** The MCP stdio transport requires that stdout is used *only* for JSON-RPC messages. Any other output (logging, greeting banners, debug info) must go to stderr. This is a bug in the MCP server implementations, not in this application.

**Workaround:** Use well-tested MCP servers that follow the protocol correctly, such as the official `@modelcontextprotocol/server-everything` reference server.

**Potential improvement:** Detect non-JSON output on stdout and surface a clearer error message to the user explaining the server is misbehaving.
