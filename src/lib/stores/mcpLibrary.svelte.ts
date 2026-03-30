import { invoke } from '@tauri-apps/api/core';
import type { Mcp, CreateMcpRequest } from '$lib/types';

class McpLibraryState {
	mcps = $state<Mcp[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	selectedType = $state<'all' | 'stdio' | 'sse' | 'http' | 'ws'>('all');

	filteredMcps = $derived.by(() => {
		let result = this.mcps;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(m) =>
					m.name.toLowerCase().includes(query) ||
					m.description?.toLowerCase().includes(query) ||
					m.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		if (this.selectedType !== 'all') {
			result = result.filter((m) => m.type === this.selectedType);
		}

		// Sort by favorites first, then by name
		return [...result].sort((a, b) => {
			if (a.isFavorite !== b.isFavorite) {
				return a.isFavorite ? -1 : 1;
			}
			return a.name.localeCompare(b.name);
		});
	});

	mcpCount = $derived.by(() => {
		let stdio = 0, sse = 0, http = 0, ws = 0;
		for (const m of this.mcps) {
			if (m.type === 'stdio') stdio++;
			else if (m.type === 'sse') sse++;
			else if (m.type === 'http') http++;
			else if (m.type === 'ws') ws++;
		}
		return { total: this.mcps.length, stdio, sse, http, ws };
	});

	async load() {
		console.log('[mcpLibrary] Loading MCPs...');
		this.isLoading = true;
		this.error = null;
		try {
			this.mcps = await invoke<Mcp[]>('get_all_mcps');
			console.log(`[mcpLibrary] Loaded ${this.mcps.length} MCPs`);
		} catch (e) {
			this.error = String(e);
			console.error('[mcpLibrary] Failed to load MCPs:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async create(request: CreateMcpRequest): Promise<Mcp> {
		console.log(`[mcpLibrary] Creating MCP: ${request.name}`);
		const mcp = await invoke<Mcp>('create_mcp', { mcp: request });
		this.mcps = [...this.mcps, mcp];
		console.log(`[mcpLibrary] Created MCP id=${mcp.id}`);
		return mcp;
	}

	async update(id: number, request: CreateMcpRequest): Promise<Mcp> {
		console.log(`[mcpLibrary] Updating MCP id=${id}: ${request.name}`);
		const mcp = await invoke<Mcp>('update_mcp', { id, mcp: request });
		this.mcps = this.mcps.map((m) => (m.id === id ? mcp : m));
		console.log(`[mcpLibrary] Updated MCP id=${id}`);
		return mcp;
	}

	async delete(id: number): Promise<void> {
		console.log(`[mcpLibrary] Deleting MCP id=${id}`);
		await invoke('delete_mcp', { id });
		this.mcps = this.mcps.filter((m) => m.id !== id);
		console.log(`[mcpLibrary] Deleted MCP id=${id}`);
	}

	async duplicate(id: number): Promise<Mcp> {
		console.log(`[mcpLibrary] Duplicating MCP id=${id}`);
		const mcp = await invoke<Mcp>('duplicate_mcp', { id });
		this.mcps = [...this.mcps, mcp];
		console.log(`[mcpLibrary] Duplicated MCP id=${id} -> id=${mcp.id}`);
		return mcp;
	}

	async toggleGlobal(id: number, enabled: boolean): Promise<void> {
		console.log(`[mcpLibrary] Toggling global MCP id=${id} enabled=${enabled}`);
		await invoke('toggle_global_mcp', { id, enabled });
		this.mcps = this.mcps.map((m) => (m.id === id ? { ...m, isEnabledGlobal: enabled } : m));
	}

	updateMcp(mcp: Mcp): void {
		this.mcps = this.mcps.map((m) => (m.id === mcp.id ? mcp : m));
	}

	getMcpById(id: number): Mcp | undefined {
		return this.mcps.find((m) => m.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}

	setTypeFilter(type: 'all' | 'stdio' | 'sse' | 'http' | 'ws') {
		this.selectedType = type;
	}
}

export const mcpLibrary = new McpLibraryState();
