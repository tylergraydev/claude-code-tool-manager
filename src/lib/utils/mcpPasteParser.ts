/**
 * Parser for MCP configurations from clipboard
 * Supports:
 * - Claude MCP CLI commands: `claude mcp add <name> -- <command> <args...>`
 * - Claude MCP add-json: `claude mcp add-json <name> '{...}'`
 * - VS Code/Claude Desktop JSON: `{"mcpServers": {...}}` or `{"command": "..."}`
 */

export interface ParsedMcp {
	name: string;
	type: 'stdio' | 'sse' | 'http';
	command?: string;
	args?: string[];
	url?: string;
	env?: Record<string, string>;
	headers?: Record<string, string>;
}

export interface ParseResult {
	success: boolean;
	mcps: ParsedMcp[];
	error?: string;
}

/**
 * Normalize multiline commands by joining lines ending with backslash
 */
function normalizeMultilineCommand(text: string): string {
	// Replace backslash + newline (with optional whitespace) with a single space
	return text
		.replace(/\\\s*\r?\n\s*/g, ' ')
		.replace(/\s+/g, ' ')
		.trim();
}

/**
 * Parse clipboard content and extract MCP configuration(s)
 */
export function parseMcpFromClipboard(text: string): ParseResult {
	let trimmed = text.trim();

	// Normalize multiline commands (backslash continuations)
	if (trimmed.includes('\\\n') || trimmed.includes('\\\r\n') || trimmed.includes('\\ \n')) {
		trimmed = normalizeMultilineCommand(trimmed);
	}

	// Try parsing as Claude MCP command
	if (trimmed.startsWith('claude mcp add')) {
		return parseClaudeMcpCommand(trimmed);
	}

	// Try parsing as JSON
	if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
		return parseJsonConfig(trimmed);
	}

	return {
		success: false,
		mcps: [],
		error: 'Unrecognized format. Paste a Claude MCP command or JSON configuration.'
	};
}

/**
 * Parse `claude mcp add` or `claude mcp add-json` commands
 */
function parseClaudeMcpCommand(text: string): ParseResult {
	// Handle add-json format: claude mcp add-json <name> '<json>'
	const addJsonMatch = text.match(/claude\s+mcp\s+add-json\s+(\S+)\s+['"]?(\{[\s\S]*\})['"]?/);
	if (addJsonMatch) {
		const name = addJsonMatch[1];
		try {
			const config = JSON.parse(addJsonMatch[2]);
			const mcp = parseServerConfig(name, config);
			return { success: true, mcps: [mcp] };
		} catch {
			return { success: false, mcps: [], error: 'Invalid JSON in add-json command' };
		}
	}

	// Handle add format: claude mcp add <name> [options] -- <command> <args...>
	// Options: -s/--scope, -e VAR=value
	const addMatch = text.match(/claude\s+mcp\s+add\s+(\S+)\s+(.*)/s);
	if (addMatch) {
		const name = addMatch[1];
		let rest = addMatch[2];

		// Extract environment variables (-e KEY=value or -e KEY="value with spaces")
		const env: Record<string, string> = {};
		// Match -e VAR=value, -e VAR="quoted value", or -e VAR='quoted value'
		const envRegex = /-e\s+(\w+)=(?:"([^"]*)"|'([^']*)'|(\S+))/g;
		let envMatch;
		while ((envMatch = envRegex.exec(rest)) !== null) {
			const key = envMatch[1];
			// Value is in group 2 (double quoted), 3 (single quoted), or 4 (unquoted)
			let value = envMatch[2] ?? envMatch[3] ?? envMatch[4];
			// Handle shell variable references like $VAR or ${VAR}
			if (value.startsWith('$')) {
				value = `\${${value.replace(/^\$\{?|\}?$/g, '')}}`;
			}
			env[key] = value;
		}

		// Remove options before the -- separator
		const dashDashIndex = rest.indexOf('--');
		if (dashDashIndex !== -1) {
			rest = rest.substring(dashDashIndex + 2).trim();
		} else {
			// Remove known options
			rest = rest.replace(/-[se]\s+\S+/g, '').replace(/--scope\s+\S+/g, '').trim();
		}

		// Parse command and args
		const parts = parseCommandLine(rest);
		if (parts.length === 0) {
			return { success: false, mcps: [], error: 'No command found in MCP add command' };
		}

		const mcp: ParsedMcp = {
			name,
			type: 'stdio',
			command: parts[0],
			args: parts.slice(1),
			env: Object.keys(env).length > 0 ? env : undefined
		};

		return { success: true, mcps: [mcp] };
	}

	return { success: false, mcps: [], error: 'Could not parse Claude MCP command' };
}

/**
 * Parse JSON configuration (VS Code, Claude Desktop, or raw server config)
 */
function parseJsonConfig(text: string): ParseResult {
	try {
		const json = JSON.parse(text);

		// Format 1: { "mcpServers": { "name": {...}, ... } }
		if (json.mcpServers && typeof json.mcpServers === 'object') {
			const mcps: ParsedMcp[] = [];
			for (const [name, config] of Object.entries(json.mcpServers)) {
				mcps.push(parseServerConfig(name, config as Record<string, unknown>));
			}
			return { success: true, mcps };
		}

		// Format 2: { "servers": { "name": {...}, ... } } (VS Code format)
		if (json.servers && typeof json.servers === 'object') {
			const mcps: ParsedMcp[] = [];
			for (const [name, config] of Object.entries(json.servers)) {
				mcps.push(parseServerConfig(name, config as Record<string, unknown>));
			}
			return { success: true, mcps };
		}

		// Format 3: Single server config { "command": "...", "args": [...] }
		if (json.command || json.url || json.type) {
			const mcp = parseServerConfig('imported-mcp', json);
			return { success: true, mcps: [mcp] };
		}

		// Format 4: { "name": { "command": "...", ... } } - single named server
		const keys = Object.keys(json);
		if (keys.length === 1 && typeof json[keys[0]] === 'object') {
			const config = json[keys[0]];
			if (config.command || config.url || config.type) {
				const mcp = parseServerConfig(keys[0], config);
				return { success: true, mcps: [mcp] };
			}
		}

		return { success: false, mcps: [], error: 'JSON does not contain MCP server configuration' };
	} catch {
		return { success: false, mcps: [], error: 'Invalid JSON' };
	}
}

/**
 * Parse a single server configuration object
 */
function parseServerConfig(name: string, config: Record<string, unknown>): ParsedMcp {
	const type = detectServerType(config);

	const mcp: ParsedMcp = { name, type };

	if (type === 'stdio') {
		if (typeof config.command === 'string') {
			mcp.command = config.command;
		} else if (Array.isArray(config.command)) {
			mcp.command = config.command[0];
			if (config.command.length > 1) {
				mcp.args = config.command.slice(1);
			}
		}

		if (Array.isArray(config.args)) {
			mcp.args = [...(mcp.args || []), ...config.args.map(String)];
		}
	} else {
		if (typeof config.url === 'string') {
			mcp.url = config.url;
		}
		if (config.headers && typeof config.headers === 'object') {
			mcp.headers = config.headers as Record<string, string>;
		}
	}

	if (config.env && typeof config.env === 'object') {
		mcp.env = config.env as Record<string, string>;
	}

	return mcp;
}

/**
 * Detect server type from config
 */
function detectServerType(config: Record<string, unknown>): 'stdio' | 'sse' | 'http' {
	if (config.type === 'sse') return 'sse';
	if (config.type === 'http') return 'http';
	if (config.url) {
		// URL-based servers are typically SSE or HTTP
		const url = String(config.url);
		if (url.includes('sse') || config.type === 'sse') return 'sse';
		return 'http';
	}
	// Default to stdio for command-based servers
	return 'stdio';
}

/**
 * Parse a command line string into parts, respecting quotes
 */
function parseCommandLine(cmdLine: string): string[] {
	const parts: string[] = [];
	let current = '';
	let inQuote: string | null = null;

	for (let i = 0; i < cmdLine.length; i++) {
		const char = cmdLine[i];

		if (inQuote) {
			if (char === inQuote) {
				inQuote = null;
			} else {
				current += char;
			}
		} else if (char === '"' || char === "'") {
			inQuote = char;
		} else if (char === ' ' || char === '\t') {
			if (current) {
				parts.push(current);
				current = '';
			}
		} else {
			current += char;
		}
	}

	if (current) {
		parts.push(current);
	}

	return parts;
}
