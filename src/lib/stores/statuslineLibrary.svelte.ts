import { invoke } from '@tauri-apps/api/core';
import type {
	StatusLine,
	CreateStatusLineRequest,
	StatusLineGalleryEntry,
	StatusLineSegment,
	StatusLineTheme
} from '$lib/types';

class StatusLineLibraryState {
	statuslines = $state<StatusLine[]>([]);
	activeStatusLine = $state<StatusLine | null>(null);
	gallery = $state<StatusLineGalleryEntry[]>([]);
	isLoading = $state(false);
	isGalleryLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredStatusLines = $derived.by(() => {
		let result = this.statuslines;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(s) =>
					s.name.toLowerCase().includes(query) ||
					s.description?.toLowerCase().includes(query) ||
					s.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		// Sort: active first, then alphabetical
		return [...result].sort((a, b) => {
			if (a.isActive !== b.isActive) return a.isActive ? -1 : 1;
			return a.name.localeCompare(b.name);
		});
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.statuslines = await invoke<StatusLine[]>('get_all_statuslines');
			this.activeStatusLine =
				(await invoke<StatusLine | null>('get_active_statusline')) ?? null;
		} catch (e) {
			this.error = String(e);
			console.error('[statuslineLibrary] Failed to load:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async create(request: CreateStatusLineRequest): Promise<StatusLine> {
		const sl = await invoke<StatusLine>('create_statusline', { request });
		this.statuslines = [...this.statuslines, sl];
		return sl;
	}

	async update(id: number, request: CreateStatusLineRequest): Promise<StatusLine> {
		const sl = await invoke<StatusLine>('update_statusline', { id, request });
		this.statuslines = this.statuslines.map((s) => (s.id === id ? sl : s));
		if (this.activeStatusLine?.id === id) {
			this.activeStatusLine = sl;
		}
		return sl;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_statusline', { id });
		this.statuslines = this.statuslines.filter((s) => s.id !== id);
		if (this.activeStatusLine?.id === id) {
			this.activeStatusLine = null;
		}
	}

	async activate(id: number): Promise<void> {
		const sl = await invoke<StatusLine>('activate_statusline', { id });
		this.statuslines = this.statuslines.map((s) => ({
			...s,
			isActive: s.id === id
		}));
		this.activeStatusLine = sl;
	}

	async deactivate(): Promise<void> {
		await invoke('deactivate_statusline');
		this.statuslines = this.statuslines.map((s) => ({ ...s, isActive: false }));
		this.activeStatusLine = null;
	}

	async loadGallery(): Promise<void> {
		this.isGalleryLoading = true;
		try {
			// Try cache first
			this.gallery = await invoke<StatusLineGalleryEntry[]>('get_statusline_gallery_cache');
			// Then fetch fresh in background
			invoke<StatusLineGalleryEntry[]>('fetch_statusline_gallery', { url: null })
				.then((entries) => {
					this.gallery = entries;
				})
				.catch((e) => {
					console.warn('[statuslineLibrary] Gallery fetch failed:', e);
				});
		} catch (e) {
			console.error('[statuslineLibrary] Failed to load gallery:', e);
		} finally {
			this.isGalleryLoading = false;
		}
	}

	async installPremade(entry: StatusLineGalleryEntry): Promise<StatusLine> {
		const sl = await invoke<StatusLine>('install_premade_statusline', { entry });
		this.statuslines = [...this.statuslines, sl];
		return sl;
	}

	async generatePreview(segments: StatusLineSegment[], theme?: StatusLineTheme): Promise<string> {
		return await invoke<string>('generate_statusline_preview', { segments, theme: theme || 'default' });
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const statuslineLibrary = new StatusLineLibraryState();
