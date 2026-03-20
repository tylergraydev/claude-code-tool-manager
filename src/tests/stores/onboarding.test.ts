import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('Onboarding Store', () => {
	let mockStorage: Record<string, string> = {};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		mockStorage = {};
		vi.stubGlobal('localStorage', {
			getItem: vi.fn((key: string) => mockStorage[key] ?? null),
			setItem: vi.fn((key: string, value: string) => { mockStorage[key] = value; }),
			removeItem: vi.fn((key: string) => { delete mockStorage[key]; })
		});
	});

	it('should initialize with default state on first run', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		expect(onboarding.isFirstRun).toBe(true);
		expect(onboarding.isDismissed).toBe(false);
		expect(onboarding.completedSteps).toEqual([]);
		expect(onboarding.isComplete).toBe(false);
		expect(onboarding.progress).toBe(0);
		expect(onboarding.showOnboarding).toBe(true);
	});

	it('should load existing state from localStorage', async () => {
		mockStorage['claude-tool-manager-onboarding'] = JSON.stringify({
			dismissed: false,
			completedSteps: ['add-project', 'add-mcp']
		});
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		expect(onboarding.completedSteps).toEqual(['add-project', 'add-mcp']);
		expect(onboarding.progress).toBe(0.5);
		expect(onboarding.isFirstRun).toBe(false);
	});

	it('should handle corrupted localStorage gracefully', async () => {
		mockStorage['claude-tool-manager-onboarding'] = 'not-json';
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		expect(onboarding.isFirstRun).toBe(true);
	});

	it('should complete a step', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.completeStep('add-project');
		expect(onboarding.completedSteps).toContain('add-project');
		expect(onboarding.progress).toBe(0.25);
		expect(localStorage.setItem).toHaveBeenCalled();
	});

	it('should not duplicate completed steps', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.completeStep('add-project');
		onboarding.completeStep('add-project');
		expect(onboarding.completedSteps.filter((s: string) => s === 'add-project')).toHaveLength(1);
	});

	it('should dismiss onboarding', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.dismiss();
		expect(onboarding.isDismissed).toBe(true);
		expect(onboarding.showOnboarding).toBe(false);
		expect(localStorage.setItem).toHaveBeenCalled();
	});

	it('should be complete when all 4 steps are done', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.completeStep('add-project');
		onboarding.completeStep('add-mcp');
		onboarding.completeStep('assign-mcp');
		onboarding.completeStep('explore-settings');
		expect(onboarding.isComplete).toBe(true);
		expect(onboarding.progress).toBe(1);
		expect(onboarding.showOnboarding).toBe(false);
	});

	it('should sync with stores - auto-complete steps', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.syncWithStores(1, 1, 1);
		expect(onboarding.completedSteps).toContain('add-project');
		expect(onboarding.completedSteps).toContain('add-mcp');
		expect(onboarding.completedSteps).toContain('assign-mcp');
	});

	it('should not sync when counts are zero', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.syncWithStores(0, 0, 0);
		expect(onboarding.completedSteps).toEqual([]);
	});

	it('should not re-sync already completed steps', async () => {
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		onboarding.completeStep('add-project');
		const setItemCallCount = (localStorage.setItem as any).mock.calls.length;
		onboarding.syncWithStores(1, 0, 0);
		// Should not have saved again since add-project was already complete
		expect((localStorage.setItem as any).mock.calls.length).toBe(setItemCallCount);
	});

	it('should handle localStorage.setItem failure gracefully', async () => {
		vi.stubGlobal('localStorage', {
			getItem: vi.fn(() => null),
			setItem: vi.fn(() => { throw new Error('quota exceeded'); }),
			removeItem: vi.fn()
		});
		const { onboarding } = await import('$lib/stores/onboarding.svelte');
		// Should not throw
		onboarding.completeStep('add-project');
		onboarding.dismiss();
	});
});
