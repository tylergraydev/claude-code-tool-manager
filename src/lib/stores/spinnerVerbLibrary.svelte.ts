import { invoke } from '@tauri-apps/api/core';
import type { SpinnerVerb } from '$lib/types';

class SpinnerVerbLibraryState {
	verbs = $state<SpinnerVerb[]>([]);
	mode = $state<'append' | 'replace'>('append');
	isLoading = $state(false);
	error = $state<string | null>(null);

	async load() {
		console.log('[spinnerVerbLibrary] Loading spinner verbs...');
		this.isLoading = true;
		this.error = null;
		try {
			this.verbs = await invoke<SpinnerVerb[]>('get_all_spinner_verbs');
			this.mode = (await invoke<string>('get_spinner_verb_mode')) as 'append' | 'replace';
			console.log(
				`[spinnerVerbLibrary] Loaded ${this.verbs.length} verbs, mode=${this.mode}`
			);
		} catch (e) {
			this.error = String(e);
			console.error('[spinnerVerbLibrary] Failed to load spinner verbs:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async create(verb: string): Promise<SpinnerVerb> {
		console.log(`[spinnerVerbLibrary] Creating verb: ${verb}`);
		const created = await invoke<SpinnerVerb>('create_spinner_verb', { verb });
		this.verbs = [...this.verbs, created];
		console.log(`[spinnerVerbLibrary] Created verb id=${created.id}`);
		return created;
	}

	async update(id: number, verb: string, isEnabled: boolean): Promise<SpinnerVerb> {
		console.log(`[spinnerVerbLibrary] Updating verb id=${id}: ${verb}`);
		const updated = await invoke<SpinnerVerb>('update_spinner_verb', { id, verb, isEnabled });
		this.verbs = this.verbs.map((v) => (v.id === id ? updated : v));
		console.log(`[spinnerVerbLibrary] Updated verb id=${id}`);
		return updated;
	}

	async delete(id: number): Promise<void> {
		console.log(`[spinnerVerbLibrary] Deleting verb id=${id}`);
		await invoke('delete_spinner_verb', { id });
		this.verbs = this.verbs.filter((v) => v.id !== id);
		console.log(`[spinnerVerbLibrary] Deleted verb id=${id}`);
	}

	async reorder(ids: number[]): Promise<void> {
		console.log(`[spinnerVerbLibrary] Reordering ${ids.length} verbs`);
		await invoke('reorder_spinner_verbs', { ids });
		// Reorder local state to match
		const verbMap = new Map(this.verbs.map((v) => [v.id, v]));
		this.verbs = ids
			.map((id, index) => {
				const verb = verbMap.get(id);
				if (verb) {
					return { ...verb, displayOrder: index };
				}
				return null;
			})
			.filter((v): v is SpinnerVerb => v !== null);
	}

	async setMode(mode: 'append' | 'replace'): Promise<void> {
		console.log(`[spinnerVerbLibrary] Setting mode to: ${mode}`);
		await invoke('set_spinner_verb_mode', { mode });
		this.mode = mode;
	}

	async sync(): Promise<void> {
		console.log('[spinnerVerbLibrary] Syncing to settings.json');
		await invoke('sync_spinner_verbs');
	}
}

export const spinnerVerbLibrary = new SpinnerVerbLibraryState();
