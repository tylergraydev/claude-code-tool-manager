import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockSessionStore = {
	projects: [] as any[]
};

vi.mock('$lib/stores/sessionStore.svelte', () => ({
	sessionStore: mockSessionStore
}));

vi.mock('$lib/types/usage', () => ({
	estimateSessionCost: vi.fn((_models: string[], input: number, output: number) => {
		return (input + output) * 0.0001;
	})
}));

describe('Comparison Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		mockSessionStore.projects = [];
	});

	describe('toggleProject', () => {
		it('should add a project', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('project-a');
			expect(comparisonStore.selectedFolders.has('project-a')).toBe(true);
		});

		it('should remove a project when toggled again', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('project-a');
			comparisonStore.toggleProject('project-a');
			expect(comparisonStore.selectedFolders.has('project-a')).toBe(false);
		});

		it('should not add more than 5 projects', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			for (let i = 0; i < 6; i++) {
				comparisonStore.toggleProject(`project-${i}`);
			}
			expect(comparisonStore.selectedFolders.size).toBe(5);
		});
	});

	describe('clearSelection', () => {
		it('should clear all selections', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('a');
			comparisonStore.toggleProject('b');
			comparisonStore.clearSelection();
			expect(comparisonStore.selectedFolders.size).toBe(0);
		});
	});

	describe('PROJECT_COLORS', () => {
		it('should export 5 colors', async () => {
			const { PROJECT_COLORS } = await import('$lib/stores/comparisonStore.svelte');
			expect(PROJECT_COLORS).toHaveLength(5);
		});
	});

	describe('selectedProjects', () => {
		it('should return projects matching selected folders', async () => {
			mockSessionStore.projects = [
				{ folderName: 'proj-a', totalInputTokens: 100, totalOutputTokens: 50, toolUsage: {}, modelsUsed: ['opus'], totalCacheReadTokens: 0, totalCacheCreationTokens: 0, earliestSession: null, latestSession: null },
				{ folderName: 'proj-b', totalInputTokens: 200, totalOutputTokens: 100, toolUsage: {}, modelsUsed: ['sonnet'], totalCacheReadTokens: 0, totalCacheCreationTokens: 0, earliestSession: null, latestSession: null },
				{ folderName: 'proj-c', totalInputTokens: 300, totalOutputTokens: 150, toolUsage: {}, modelsUsed: ['haiku'], totalCacheReadTokens: 0, totalCacheCreationTokens: 0, earliestSession: null, latestSession: null }
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');
			comparisonStore.toggleProject('proj-c');

			expect(comparisonStore.selectedProjects).toHaveLength(2);
			expect(comparisonStore.selectedProjects[0].folderName).toBe('proj-a');
			expect(comparisonStore.selectedProjects[1].folderName).toBe('proj-c');
		});

		it('should return empty when nothing selected', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			expect(comparisonStore.selectedProjects).toEqual([]);
		});
	});

	describe('comparisonData', () => {
		it('should compute comparison data for selected projects', async () => {
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 1000,
					totalOutputTokens: 500,
					toolUsage: { Read: 10, Write: 5, Bash: 3 },
					modelsUsed: ['opus'],
					totalCacheReadTokens: 0,
					totalCacheCreationTokens: 0,
					earliestSession: '2025-01-01T00:00:00Z',
					latestSession: '2025-01-15T00:00:00Z'
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');

			expect(comparisonStore.comparisonData).toHaveLength(1);
			const data = comparisonStore.comparisonData[0];
			expect(data.totalTokens).toBe(1500);
			expect(data.topTools).toEqual([['Read', 10], ['Write', 5], ['Bash', 3]]);
			expect(data.color).toBe('#3b82f6'); // first color
		});

		it('should assign fallback color when more than 5 projects', async () => {
			// Create 6 projects
			const projects = [];
			for (let i = 0; i < 6; i++) {
				projects.push({
					folderName: `proj-${i}`,
					totalInputTokens: 100,
					totalOutputTokens: 50,
					toolUsage: {},
					modelsUsed: ['opus'],
					totalCacheReadTokens: 0,
					totalCacheCreationTokens: 0,
					earliestSession: null,
					latestSession: null
				});
			}
			mockSessionStore.projects = projects;

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			// Select 5 (max)
			for (let i = 0; i < 5; i++) {
				comparisonStore.toggleProject(`proj-${i}`);
			}

			// All 5 should have colors from PROJECT_COLORS
			expect(comparisonStore.comparisonData).toHaveLength(5);
		});

		it('should handle projects with dateRange formatting', async () => {
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 100,
					totalOutputTokens: 50,
					toolUsage: {},
					modelsUsed: [],
					totalCacheReadTokens: 0,
					totalCacheCreationTokens: 0,
					earliestSession: '2025-01-01T00:00:00Z',
					latestSession: '2025-02-15T00:00:00Z'
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');

			const data = comparisonStore.comparisonData[0];
			// Should contain formatted dates with en-dash separator
			expect(data.dateRange).toContain('\u2013'); // en-dash
		});

		it('should handle null dates in dateRange', async () => {
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 100,
					totalOutputTokens: 50,
					toolUsage: {},
					modelsUsed: [],
					totalCacheReadTokens: 0,
					totalCacheCreationTokens: 0,
					earliestSession: null,
					latestSession: null
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');

			const data = comparisonStore.comparisonData[0];
			expect(data.dateRange).toContain('?');
		});
	});

	describe('allTools', () => {
		it('should aggregate and sort tools across projects', async () => {
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 100, totalOutputTokens: 50,
					toolUsage: { Read: 10, Write: 5 },
					modelsUsed: [], totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
					earliestSession: null, latestSession: null
				},
				{
					folderName: 'proj-b',
					totalInputTokens: 200, totalOutputTokens: 100,
					toolUsage: { Read: 20, Bash: 15 },
					modelsUsed: [], totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
					earliestSession: null, latestSession: null
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');
			comparisonStore.toggleProject('proj-b');

			// Should be sorted by total count descending
			expect(comparisonStore.allTools[0]).toBe('Read'); // 30 total
			expect(comparisonStore.allTools[1]).toBe('Bash'); // 15
			expect(comparisonStore.allTools[2]).toBe('Write'); // 5
		});

		it('should limit to top 10 tools', async () => {
			const toolUsage: Record<string, number> = {};
			for (let i = 0; i < 15; i++) {
				toolUsage[`tool-${i}`] = 15 - i;
			}
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 100, totalOutputTokens: 50,
					toolUsage,
					modelsUsed: [], totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
					earliestSession: null, latestSession: null
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');

			expect(comparisonStore.allTools).toHaveLength(10);
		});

		it('should return empty when no selection', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			expect(comparisonStore.allTools).toEqual([]);
		});
	});

	describe('allModels', () => {
		it('should collect unique models from selected projects', async () => {
			mockSessionStore.projects = [
				{
					folderName: 'proj-a',
					totalInputTokens: 100, totalOutputTokens: 50,
					toolUsage: {},
					modelsUsed: ['opus', 'sonnet'],
					totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
					earliestSession: null, latestSession: null
				},
				{
					folderName: 'proj-b',
					totalInputTokens: 200, totalOutputTokens: 100,
					toolUsage: {},
					modelsUsed: ['sonnet', 'haiku'],
					totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
					earliestSession: null, latestSession: null
				}
			];

			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			comparisonStore.toggleProject('proj-a');
			comparisonStore.toggleProject('proj-b');

			expect(comparisonStore.allModels).toEqual(['haiku', 'opus', 'sonnet']);
		});

		it('should return empty when no selection', async () => {
			const { comparisonStore } = await import('$lib/stores/comparisonStore.svelte');
			expect(comparisonStore.allModels).toEqual([]);
		});
	});
});
