import { invoke } from '@tauri-apps/api/core';

export interface ClaudeJsonMcp {
	name: string;
	type: 'stdio' | 'sse' | 'http';
	command?: string;
	args?: string[];
	url?: string;
	headers?: Record<string, string>;
	env?: Record<string, string>;
	projectPath?: string;
	isEnabled: boolean;
}

export interface ClaudeJsonProject {
	path: string;
	mcps: ClaudeJsonMcp[];
}

class ClaudeJsonState {
	mcps = $state<ClaudeJsonMcp[]>([]);
	projects = $state<ClaudeJsonProject[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);

	// Get global MCPs (no project path)
	globalMcps = $derived(this.mcps.filter((m) => !m.projectPath));

	// Group MCPs by project
	mcpsByProject = $derived.by(() => {
		const map = new Map<string, ClaudeJsonMcp[]>();
		for (const mcp of this.mcps) {
			if (mcp.projectPath) {
				const existing = map.get(mcp.projectPath) ?? [];
				existing.push(mcp);
				map.set(mcp.projectPath, existing);
			}
		}
		return map;
	});

	async loadAll() {
		this.isLoading = true;
		this.error = null;
		try {
			const [mcps, projects] = await Promise.all([
				invoke<ClaudeJsonMcp[]>('get_claude_json_mcps'),
				invoke<ClaudeJsonProject[]>('get_claude_json_projects')
			]);
			this.mcps = mcps;
			this.projects = projects;
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load claude.json data:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async toggleMcp(projectPath: string, mcpName: string, enabled: boolean) {
		try {
			await invoke('toggle_mcp_in_claude_json', { projectPath, mcpName, enabled });
			// Update local state
			this.mcps = this.mcps.map((m) =>
				m.projectPath === projectPath && m.name === mcpName ? { ...m, isEnabled: enabled } : m
			);
		} catch (e) {
			console.error('Failed to toggle MCP:', e);
			throw e;
		}
	}

	async removeMcpFromProject(projectPath: string, mcpName: string) {
		try {
			await invoke('remove_mcp_from_claude_json', { projectPath, mcpName });
			this.mcps = this.mcps.filter(
				(m) => !(m.projectPath === projectPath && m.name === mcpName)
			);
		} catch (e) {
			console.error('Failed to remove MCP:', e);
			throw e;
		}
	}

	async removeGlobalMcp(mcpName: string) {
		try {
			await invoke('remove_global_mcp_from_claude_json', { mcpName });
			this.mcps = this.mcps.filter((m) => !(m.name === mcpName && !m.projectPath));
		} catch (e) {
			console.error('Failed to remove global MCP:', e);
			throw e;
		}
	}
}

export const claudeJson = new ClaudeJsonState();
