import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createMockProfile, resetIdCounter } from '../factories';

describe('Profile Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetIdCounter();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load profiles', async () => {
			const mockProfiles = [
				createMockProfile({ id: 1, name: 'dev' }),
				createMockProfile({ id: 2, name: 'prod' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();

			expect(profileLibrary.profiles).toHaveLength(2);
			expect(profileLibrary.profiles[0].name).toBe('dev');
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			const loadPromise = profileLibrary.load();

			expect(profileLibrary.isLoading).toBe(true);

			resolveInvoke!([]);
			await loadPromise;

			expect(profileLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load error'));

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();

			expect(profileLibrary.error).toContain('Load error');
			expect(profileLibrary.isLoading).toBe(false);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();

			expect(profileLibrary.profiles).toHaveLength(0);
		});
	});

	describe('filteredProfiles', () => {
		it('should filter by name', async () => {
			const mockProfiles = [
				createMockProfile({ id: 1, name: 'development' }),
				createMockProfile({ id: 2, name: 'production' }),
				createMockProfile({ id: 3, name: 'staging' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			profileLibrary.setSearch('dev');

			expect(profileLibrary.filteredProfiles).toHaveLength(1);
			expect(profileLibrary.filteredProfiles[0].name).toBe('development');
		});

		it('should filter by description', async () => {
			const mockProfiles = [
				createMockProfile({ id: 1, name: 'p1', description: 'For testing' }),
				createMockProfile({ id: 2, name: 'p2', description: 'For deployment' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			profileLibrary.setSearch('testing');

			expect(profileLibrary.filteredProfiles).toHaveLength(1);
		});

		it('should sort alphabetically', async () => {
			const mockProfiles = [
				createMockProfile({ id: 1, name: 'charlie' }),
				createMockProfile({ id: 2, name: 'alpha' }),
				createMockProfile({ id: 3, name: 'bravo' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();

			expect(profileLibrary.filteredProfiles[0].name).toBe('alpha');
			expect(profileLibrary.filteredProfiles[1].name).toBe('bravo');
			expect(profileLibrary.filteredProfiles[2].name).toBe('charlie');
		});

		it('should return all profiles when search is empty', async () => {
			const mockProfiles = [
				createMockProfile({ id: 1 }),
				createMockProfile({ id: 2 })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			profileLibrary.setSearch('');

			expect(profileLibrary.filteredProfiles).toHaveLength(2);
		});
	});

	describe('create', () => {
		it('should create profile and add to local state', async () => {
			const newProfile = createMockProfile({ id: 10, name: 'new-profile' });
			vi.mocked(invoke).mockResolvedValueOnce(newProfile);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			const result = await profileLibrary.create({ name: 'new-profile' });

			expect(result.id).toBe(10);
			expect(profileLibrary.profiles).toHaveLength(1);
		});
	});

	describe('update', () => {
		it('should update profile in local state', async () => {
			const initial = [createMockProfile({ id: 1, name: 'old-name' })];
			const updated = createMockProfile({ id: 1, name: 'new-name' });

			vi.mocked(invoke)
				.mockResolvedValueOnce(initial)
				.mockResolvedValueOnce(updated);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			await profileLibrary.update(1, { name: 'new-name' });

			expect(profileLibrary.profiles[0].name).toBe('new-name');
		});
	});

	describe('delete', () => {
		it('should remove profile from local state', async () => {
			const profiles = [
				createMockProfile({ id: 1 }),
				createMockProfile({ id: 2 })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(profiles)
				.mockResolvedValueOnce(undefined);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			await profileLibrary.delete(1);

			expect(profileLibrary.profiles).toHaveLength(1);
			expect(profileLibrary.profiles[0].id).toBe(2);
		});

		it('should clear activeProfile if deleted profile was active', async () => {
			const activeProfile = createMockProfile({ id: 1, isActive: true });
			const profiles = [activeProfile];

			vi.mocked(invoke)
				.mockResolvedValueOnce(profiles)
				.mockResolvedValueOnce(activeProfile) // loadActiveProfile
				.mockResolvedValueOnce(undefined); // delete

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			await profileLibrary.loadActiveProfile();
			await profileLibrary.delete(1);

			expect(profileLibrary.activeProfile).toBeNull();
			expect(profileLibrary.profiles).toHaveLength(0);
		});
	});

	describe('activate', () => {
		it('should activate profile and update local state', async () => {
			const profiles = [
				createMockProfile({ id: 1, isActive: false }),
				createMockProfile({ id: 2, isActive: false })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(profiles)
				.mockResolvedValueOnce(undefined); // activate

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			await profileLibrary.activate(1);

			expect(profileLibrary.profiles[0].isActive).toBe(true);
			expect(profileLibrary.profiles[1].isActive).toBe(false);
			expect(profileLibrary.activeProfile?.id).toBe(1);
		});
	});

	describe('deactivate', () => {
		it('should deactivate all profiles', async () => {
			const profiles = [
				createMockProfile({ id: 1, isActive: true }),
				createMockProfile({ id: 2, isActive: false })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(profiles)
				.mockResolvedValueOnce(undefined); // deactivate

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.load();
			await profileLibrary.deactivate();

			expect(profileLibrary.profiles.every((p) => !p.isActive)).toBe(true);
			expect(profileLibrary.activeProfile).toBeNull();
		});
	});

	describe('loadActiveProfile', () => {
		it('should load active profile', async () => {
			const active = createMockProfile({ id: 1, isActive: true });
			vi.mocked(invoke).mockResolvedValueOnce(active);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.loadActiveProfile();

			expect(profileLibrary.activeProfile?.id).toBe(1);
		});

		it('should handle null active profile', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(null);

			const { profileLibrary } = await import('$lib/stores/profileLibrary.svelte');
			await profileLibrary.loadActiveProfile();

			expect(profileLibrary.activeProfile).toBeNull();
		});
	});
});
