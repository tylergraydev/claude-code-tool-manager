import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

describe('Container Library Store', () => {
	it('should have initial state', async () => {
		const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');

		expect(containerLibrary.containers).toEqual([]);
		expect(containerLibrary.isLoading).toBe(false);
		expect(containerLibrary.error).toBeNull();
	});
});
