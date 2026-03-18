import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Insights Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('loadReport', () => {
		it('should load report successfully', async () => {
			const mockReport = { exists: true, htmlContent: '<h1>Report</h1>', filePath: '/path/report.html' };
			vi.mocked(invoke).mockResolvedValueOnce(mockReport);

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadReport();

			expect(insightsStore.reportInfo).toEqual(mockReport);
			expect(insightsStore.reportExists).toBe(true);
			expect(insightsStore.reportHtml).toBe('<h1>Report</h1>');
			expect(insightsStore.reportFilePath).toBe('/path/report.html');
			expect(insightsStore.isLoadingReport).toBe(false);
		});

		it('should handle report load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadReport();

			expect(insightsStore.reportError).toBe('Error: fail');
			expect(insightsStore.isLoadingReport).toBe(false);
		});
	});

	describe('loadFacets', () => {
		it('should load facets successfully', async () => {
			const mockFacets = {
				exists: true,
				facets: [
					{ outcome: 'success', claudeHelpfulness: 'very_helpful', frictionCounts: { slow: 2 } },
					{ outcome: 'success', claudeHelpfulness: 'helpful', frictionCounts: { slow: 1, error: 3 } },
					{ outcome: 'failure', claudeHelpfulness: null, frictionCounts: {} }
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockFacets);

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadFacets();

			expect(insightsStore.facetsInfo).toEqual(mockFacets);
			expect(insightsStore.facetsExist).toBe(true);
			expect(insightsStore.facets).toHaveLength(3);
		});

		it('should handle facets load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('facet fail'));

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadFacets();

			expect(insightsStore.facetsError).toBe('Error: facet fail');
			expect(insightsStore.isLoadingFacets).toBe(false);
		});
	});

	describe('load', () => {
		it('should load both report and facets', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce({ exists: false })
				.mockResolvedValueOnce({ exists: false, facets: [] });

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.load();

			expect(invoke).toHaveBeenCalledTimes(2);
		});
	});

	describe('derived counts', () => {
		it('should compute outcomeCounts', async () => {
			const mockFacets = {
				exists: true,
				facets: [
					{ outcome: 'success', claudeHelpfulness: null, frictionCounts: {} },
					{ outcome: 'success', claudeHelpfulness: null, frictionCounts: {} },
					{ outcome: 'failure', claudeHelpfulness: null, frictionCounts: {} },
					{ outcome: null, claudeHelpfulness: null, frictionCounts: {} }
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockFacets);

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadFacets();

			expect(insightsStore.outcomeCounts).toEqual({ success: 2, failure: 1 });
		});

		it('should compute helpfulnessCounts', async () => {
			const mockFacets = {
				exists: true,
				facets: [
					{ outcome: null, claudeHelpfulness: 'very_helpful', frictionCounts: {} },
					{ outcome: null, claudeHelpfulness: 'very_helpful', frictionCounts: {} },
					{ outcome: null, claudeHelpfulness: 'not_helpful', frictionCounts: {} }
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockFacets);

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadFacets();

			expect(insightsStore.helpfulnessCounts).toEqual({ very_helpful: 2, not_helpful: 1 });
		});

		it('should compute aggregatedFrictionCounts', async () => {
			const mockFacets = {
				exists: true,
				facets: [
					{ outcome: null, claudeHelpfulness: null, frictionCounts: { slow: 2, error: 1 } },
					{ outcome: null, claudeHelpfulness: null, frictionCounts: { slow: 3 } }
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockFacets);

			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			await insightsStore.loadFacets();

			expect(insightsStore.aggregatedFrictionCounts).toEqual({ slow: 5, error: 1 });
		});
	});

	describe('default derived values', () => {
		it('should return defaults when no data loaded', async () => {
			const { insightsStore } = await import('$lib/stores/insightsStore.svelte');
			expect(insightsStore.reportExists).toBe(false);
			expect(insightsStore.reportHtml).toBeNull();
			expect(insightsStore.reportFilePath).toBe('');
			expect(insightsStore.facetsExist).toBe(false);
			expect(insightsStore.facets).toEqual([]);
			expect(insightsStore.isLoading).toBe(false);
		});
	});
});
