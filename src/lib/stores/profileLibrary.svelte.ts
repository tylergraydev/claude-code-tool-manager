import { invoke } from '@tauri-apps/api/core';
import type { Profile, CreateProfileRequest, ProfileWithItems } from '$lib/types';

class ProfileLibraryState {
	profiles = $state<Profile[]>([]);
	activeProfile = $state<Profile | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredProfiles = $derived.by(() => {
		let result = this.profiles;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(p) =>
					p.name.toLowerCase().includes(query) ||
					p.description?.toLowerCase().includes(query)
			);
		}

		return [...result].sort((a, b) => a.name.localeCompare(b.name));
	});

	async load() {
		console.log('[profileLibrary] Loading profiles...');
		this.isLoading = true;
		this.error = null;
		try {
			this.profiles = await invoke<Profile[]>('get_all_profiles');
			console.log(`[profileLibrary] Loaded ${this.profiles.length} profiles`);
		} catch (e) {
			this.error = String(e);
			console.error('[profileLibrary] Failed to load profiles:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadActiveProfile() {
		try {
			this.activeProfile = await invoke<Profile | null>('get_active_profile');
		} catch (e) {
			console.error('[profileLibrary] Failed to load active profile:', e);
		}
	}

	async getProfile(id: number): Promise<ProfileWithItems> {
		return await invoke<ProfileWithItems>('get_profile', { id });
	}

	async create(request: CreateProfileRequest): Promise<Profile> {
		console.log(`[profileLibrary] Creating profile: ${request.name}`);
		const profile = await invoke<Profile>('create_profile', { request });
		this.profiles = [...this.profiles, profile];
		console.log(`[profileLibrary] Created profile id=${profile.id}`);
		return profile;
	}

	async update(id: number, request: CreateProfileRequest): Promise<Profile> {
		console.log(`[profileLibrary] Updating profile id=${id}: ${request.name}`);
		const profile = await invoke<Profile>('update_profile', { id, request });
		this.profiles = this.profiles.map((p) => (p.id === id ? profile : p));
		console.log(`[profileLibrary] Updated profile id=${id}`);
		return profile;
	}

	async delete(id: number): Promise<void> {
		console.log(`[profileLibrary] Deleting profile id=${id}`);
		await invoke('delete_profile', { id });
		this.profiles = this.profiles.filter((p) => p.id !== id);
		if (this.activeProfile?.id === id) {
			this.activeProfile = null;
		}
		console.log(`[profileLibrary] Deleted profile id=${id}`);
	}

	async captureFromCurrent(profileId: number): Promise<ProfileWithItems> {
		console.log(`[profileLibrary] Capturing current config into profile id=${profileId}`);
		const result = await invoke<ProfileWithItems>('capture_profile_from_current', {
			profileId
		});
		// Update the profile in local state
		this.profiles = this.profiles.map((p) =>
			p.id === profileId ? result.profile : p
		);
		return result;
	}

	async activate(id: number): Promise<void> {
		console.log(`[profileLibrary] Activating profile id=${id}`);
		await invoke('activate_profile', { id });
		// Update local state
		this.profiles = this.profiles.map((p) => ({
			...p,
			isActive: p.id === id
		}));
		this.activeProfile = this.profiles.find((p) => p.id === id) || null;
		console.log(`[profileLibrary] Activated profile id=${id}`);
	}

	async deactivate(): Promise<void> {
		console.log('[profileLibrary] Deactivating all profiles');
		await invoke('deactivate_profile');
		this.profiles = this.profiles.map((p) => ({ ...p, isActive: false }));
		this.activeProfile = null;
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const profileLibrary = new ProfileLibraryState();
