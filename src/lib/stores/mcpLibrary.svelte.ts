import { invoke } from '@tauri-apps/api/core';
import type { Mcp, CreateMcpRequest } from '$lib/types';

class McpLibraryState {
	mcps = $state<Mcp[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	selectedType = $state<'all' | 'stdio' | 'sse' | 'http'>('all');

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

		return result;
	});

	mcpCount = $derived.by(() => ({
		total: this.mcps.length,
		stdio: this.mcps.filter((m) => m.type === 'stdio').length,
		sse: this.mcps.filter((m) => m.type === 'sse').length,
		http: this.mcps.filter((m) => m.type === 'http').length
	}));

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.mcps = await invoke<Mcp[]>('get_all_mcps');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load MCPs:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async create(request: CreateMcpRequest): Promise<Mcp> {
		const mcp = await invoke<Mcp>('create_mcp', { mcp: request });
		this.mcps = [...this.mcps, mcp];
		return mcp;
	}

	async update(id: number, request: CreateMcpRequest): Promise<Mcp> {
		const mcp = await invoke<Mcp>('update_mcp', { id, mcp: request });
		this.mcps = this.mcps.map((m) => (m.id === id ? mcp : m));
		return mcp;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_mcp', { id });
		this.mcps = this.mcps.filter((m) => m.id !== id);
	}

	async duplicate(id: number): Promise<Mcp> {
		const mcp = await invoke<Mcp>('duplicate_mcp', { id });
		this.mcps = [...this.mcps, mcp];
		return mcp;
	}

	async toggleGlobal(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_mcp', { id, enabled });
		this.mcps = this.mcps.map((m) => (m.id === id ? { ...m, isEnabledGlobal: enabled } : m));
	}

	getMcpById(id: number): Mcp | undefined {
		return this.mcps.find((m) => m.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}

	setTypeFilter(type: 'all' | 'stdio' | 'sse' | 'http') {
		this.selectedType = type;
	}
}

export const mcpLibrary = new McpLibraryState();
