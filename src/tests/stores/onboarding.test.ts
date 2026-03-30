import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

describe('Onboarding Store', () => {
	it('should have initial state', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');

		expect(onboarding.completedSteps).toEqual([]);
		expect(onboarding.dismissed).toBe(false);
		expect(onboarding.isFirstRun).toBe(true);
		expect(onboarding.showOnboarding).toBe(true);
	});

	it('should complete a step', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');

		onboarding.completeStep('explore-settings');
		expect(onboarding.completedSteps).toContain('explore-settings');
	});

	it('should not duplicate steps', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');

		onboarding.completeStep('add-project');
		onboarding.completeStep('add-project');
		expect(onboarding.completedSteps.filter((s: string) => s === 'add-project')).toHaveLength(1);
	});

	it('should dismiss', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');

		onboarding.dismiss();
		expect(onboarding.dismissed).toBe(true);
	});
});
