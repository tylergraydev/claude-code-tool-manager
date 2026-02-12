import { describe, it, expect } from 'vitest';
import { parseMcpFromClipboard } from '$lib/utils/mcpPasteParser';

describe('MCP Paste Parser', () => {
	describe('Claude MCP commands', () => {
		it('should parse simple claude mcp add command', () => {
			const cmd = 'claude mcp add filesystem -- npx -y @modelcontextprotocol/server-filesystem ~/Documents';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps).toHaveLength(1);
			expect(result.mcps[0].name).toBe('filesystem');
			expect(result.mcps[0].type).toBe('stdio');
			expect(result.mcps[0].command).toBe('npx');
			expect(result.mcps[0].args).toContain('-y');
			expect(result.mcps[0].args).toContain('@modelcontextprotocol/server-filesystem');
			expect(result.mcps[0].args).toContain('~/Documents');
		});

		it('should parse claude mcp add with environment variables', () => {
			const cmd = 'claude mcp add github -e GITHUB_TOKEN=$TOKEN -- npx -y @modelcontextprotocol/server-github';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('github');
			expect(result.mcps[0].env).toBeDefined();
			expect(result.mcps[0].env?.GITHUB_TOKEN).toContain('TOKEN');
		});

		it('should parse multiline command with backslash continuations', () => {
			const cmd = `claude mcp add code-search \\
-e QDRANT_URL="http://localhost:6333" \\
-e COLLECTION_NAME="code-repository" \\
-e EMBEDDING_MODEL="sentence-transformers/all-MiniLM-L6-v2" \\
-e TOOL_STORE_DESCRIPTION="Store code snippets with descriptions." \\
-- uvx mcp-server-qdrant`;
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('code-search');
			expect(result.mcps[0].command).toBe('uvx');
			expect(result.mcps[0].args).toEqual(['mcp-server-qdrant']);
			expect(result.mcps[0].env?.QDRANT_URL).toBe('http://localhost:6333');
			expect(result.mcps[0].env?.COLLECTION_NAME).toBe('code-repository');
			expect(result.mcps[0].env?.TOOL_STORE_DESCRIPTION).toBe('Store code snippets with descriptions.');
		});

		it('should parse command with quoted env values containing spaces', () => {
			const cmd = 'claude mcp add test -e DESC="Hello World" -- node server.js';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].env?.DESC).toBe('Hello World');
		});

		it('should parse claude mcp add-json command', () => {
			const cmd = `claude mcp add-json myserver '{"command":"npx","args":["-y","@package/server"]}'`;
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('myserver');
			expect(result.mcps[0].command).toBe('npx');
			expect(result.mcps[0].args).toEqual(['-y', '@package/server']);
		});
	});

	describe('JSON configurations', () => {
		it('should parse VS Code mcpServers format', () => {
			const json = JSON.stringify({
				mcpServers: {
					filesystem: {
						command: 'npx',
						args: ['-y', '@modelcontextprotocol/server-filesystem', '~/Documents']
					}
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps).toHaveLength(1);
			expect(result.mcps[0].name).toBe('filesystem');
			expect(result.mcps[0].command).toBe('npx');
		});

		it('should parse VS Code servers format', () => {
			const json = JSON.stringify({
				servers: {
					'my-server': {
						command: 'node',
						args: ['server.js']
					}
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('my-server');
		});

		it('should parse single server config', () => {
			const json = JSON.stringify({
				command: 'python',
				args: ['-m', 'mcp_server'],
				env: { DEBUG: '1' }
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].command).toBe('python');
			expect(result.mcps[0].env?.DEBUG).toBe('1');
		});

		it('should parse HTTP/SSE server config', () => {
			const json = JSON.stringify({
				mcpServers: {
					'remote-server': {
						type: 'sse',
						url: 'https://mcp.example.com/sse'
					}
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].type).toBe('sse');
			expect(result.mcps[0].url).toBe('https://mcp.example.com/sse');
		});

		it('should parse multiple servers', () => {
			const json = JSON.stringify({
				mcpServers: {
					server1: { command: 'cmd1', args: [] },
					server2: { command: 'cmd2', args: [] }
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps).toHaveLength(2);
		});
	});

	describe('edge cases', () => {
		it('should parse nested JSON with single named server', () => {
			const json = JSON.stringify({
				'my-server': {
					command: 'node',
					args: ['server.js']
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('my-server');
			expect(result.mcps[0].command).toBe('node');
			expect(result.mcps[0].args).toEqual(['server.js']);
		});

		it('should parse command with multiple -e flags', () => {
			const cmd = 'claude mcp add multi-env -e KEY1=val1 -e KEY2="val 2" -e KEY3=val3 -- node server.js';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].env?.KEY1).toBe('val1');
			expect(result.mcps[0].env?.KEY2).toBe('val 2');
			expect(result.mcps[0].env?.KEY3).toBe('val3');
		});

		it('should handle shell variable references in env values', () => {
			const cmd = 'claude mcp add test -e API_KEY=$MY_TOKEN -- npx server';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].env?.API_KEY).toContain('MY_TOKEN');
		});

		it('should parse command with no args after --', () => {
			const cmd = 'claude mcp add simple -- python';
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].command).toBe('python');
			expect(result.mcps[0].args).toEqual([]);
		});

		it('should parse HTTP server with headers', () => {
			const json = JSON.stringify({
				mcpServers: {
					'api-server': {
						type: 'http',
						url: 'https://api.example.com/mcp',
						headers: {
							Authorization: 'Bearer token123',
							'X-Custom': 'value'
						}
					}
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].type).toBe('http');
			expect(result.mcps[0].headers?.Authorization).toBe('Bearer token123');
			expect(result.mcps[0].headers?.['X-Custom']).toBe('value');
		});

		it('should parse add-json with env variables', () => {
			const cmd = `claude mcp add-json myserver '{"command":"npx","args":["-y","@pkg/srv"],"env":{"TOKEN":"abc"}}'`;
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].name).toBe('myserver');
			expect(result.mcps[0].env?.TOKEN).toBe('abc');
		});

		it('should handle URL-based server without explicit type as http', () => {
			const json = JSON.stringify({
				command: undefined,
				url: 'https://api.example.com/mcp'
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].type).toBe('http');
			expect(result.mcps[0].url).toBe('https://api.example.com/mcp');
		});

		it('should detect SSE type from URL containing sse', () => {
			const json = JSON.stringify({
				mcpServers: {
					'sse-server': {
						url: 'https://mcp.example.com/sse'
					}
				}
			});
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps[0].type).toBe('sse');
		});

		it('should parse command with single-quoted env values', () => {
			const cmd = "claude mcp add test -e DESC='Hello World' -- node server.js";
			const result = parseMcpFromClipboard(cmd);

			expect(result.success).toBe(true);
			expect(result.mcps[0].env?.DESC).toBe('Hello World');
		});

		it('should handle empty mcpServers object', () => {
			const json = JSON.stringify({ mcpServers: {} });
			const result = parseMcpFromClipboard(json);

			expect(result.success).toBe(true);
			expect(result.mcps).toHaveLength(0);
		});
	});

	describe('error handling', () => {
		it('should return error for invalid JSON', () => {
			const result = parseMcpFromClipboard('{invalid json}');
			expect(result.success).toBe(false);
			expect(result.error).toContain('Invalid JSON');
		});

		it('should return error for unrecognized format', () => {
			const result = parseMcpFromClipboard('random text');
			expect(result.success).toBe(false);
			expect(result.error).toContain('Unrecognized format');
		});

		it('should return error for JSON without MCP config', () => {
			const json = JSON.stringify({ foo: 'bar' });
			const result = parseMcpFromClipboard(json);
			expect(result.success).toBe(false);
		});
	});
});
