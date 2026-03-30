import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

describe('Onboarding Store', () => {
	it('should have initial state', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');

		expect(onboarding.completed).toBe(false);
		expect(onboarding.currentStep).toBe(0);
	});
});
