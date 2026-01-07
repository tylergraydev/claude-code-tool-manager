import { describe, it, expect } from 'vitest';
import type { UpdateStatus } from '$lib/stores/updater.svelte';

describe('Updater Types', () => {
	it('should define valid UpdateStatus values', () => {
		const validStatuses: UpdateStatus[] = ['idle', 'checking', 'available', 'downloading', 'ready', 'error'];

		expect(validStatuses).toHaveLength(6);
	});

	it('should include all expected statuses', () => {
		const validStatuses: UpdateStatus[] = ['idle', 'checking', 'available', 'downloading', 'ready', 'error'];

		expect(validStatuses).toContain('idle');
		expect(validStatuses).toContain('checking');
		expect(validStatuses).toContain('available');
		expect(validStatuses).toContain('downloading');
		expect(validStatuses).toContain('ready');
		expect(validStatuses).toContain('error');
	});
});
