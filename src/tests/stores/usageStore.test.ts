import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

vi.mock('$lib/types/usage', () => ({
	estimateModelCost: vi.fn((modelId: string, input: number, output: number) => {
		return (input + output) * 0.001;
	})
}));

describe('Usage Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('load', () => {
		it('should load usage stats successfully', async () => {
			const mockData = {
				filePath: '/path/to/stats.json',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.data).toEqual(mockData);
			expect(usageStore.exists).toBe(true);
			expect(usageStore.filePath).toBe('/path/to/stats.json');
			expect(usageStore.isLoading).toBe(false);
			expect(usageStore.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			const loadPromise = usageStore.load();

			expect(usageStore.isLoading).toBe(true);

			resolveInvoke!({ filePath: '', exists: false, stats: null });
			await loadPromise;

			expect(usageStore.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.error).toContain('Failed to load');
			expect(usageStore.isLoading).toBe(false);
		});
	});

	describe('default state', () => {
		it('should have correct defaults', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.data).toBeNull();
			expect(usageStore.isLoading).toBe(false);
			expect(usageStore.error).toBeNull();
			expect(usageStore.dateRange).toBe('30d');
			expect(usageStore.stats).toBeNull();
			expect(usageStore.exists).toBe(false);
			expect(usageStore.filePath).toBe('');
		});
	});

	describe('setDateRange', () => {
		it('should set date range', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			usageStore.setDateRange('7d');
			expect(usageStore.dateRange).toBe('7d');
			usageStore.setDateRange('all');
			expect(usageStore.dateRange).toBe('all');
			usageStore.setDateRange('30d');
			expect(usageStore.dateRange).toBe('30d');
		});
	});

	describe('filteredDailyActivity', () => {
		it('should return all activity when dateRange is all', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [
						{ date: '2020-01-01', messageCount: 1, sessionCount: 1, toolCallCount: 5 },
						{ date: '2025-01-01', messageCount: 2, sessionCount: 1, toolCallCount: 10 }
					],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();
			usageStore.setDateRange('all');

			expect(usageStore.filteredDailyActivity).toHaveLength(2);
		});

		it('should filter activity by 7d range', async () => {
			const today = new Date();
			const recentDate = new Date(today);
			recentDate.setDate(today.getDate() - 3);
			const oldDate = new Date(today);
			oldDate.setDate(today.getDate() - 10);

			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [
						{ date: oldDate.toISOString().slice(0, 10), messageCount: 1, sessionCount: 1, toolCallCount: 5 },
						{ date: recentDate.toISOString().slice(0, 10), messageCount: 2, sessionCount: 1, toolCallCount: 10 }
					],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();
			usageStore.setDateRange('7d');

			expect(usageStore.filteredDailyActivity).toHaveLength(1);
			expect(usageStore.filteredDailyActivity[0].messageCount).toBe(2);
		});

		it('should filter activity by 30d range', async () => {
			const today = new Date();
			const recentDate = new Date(today);
			recentDate.setDate(today.getDate() - 15);
			const oldDate = new Date(today);
			oldDate.setDate(today.getDate() - 45);

			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [
						{ date: oldDate.toISOString().slice(0, 10), messageCount: 1, sessionCount: 1, toolCallCount: 5 },
						{ date: recentDate.toISOString().slice(0, 10), messageCount: 2, sessionCount: 1, toolCallCount: 10 }
					],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();
			usageStore.setDateRange('30d');

			expect(usageStore.filteredDailyActivity).toHaveLength(1);
			expect(usageStore.filteredDailyActivity[0].messageCount).toBe(2);
		});

		it('should return empty array when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.filteredDailyActivity).toEqual([]);
		});
	});

	describe('filteredDailyTokens', () => {
		it('should filter tokens by date range', async () => {
			const today = new Date();
			const recentDate = new Date(today);
			recentDate.setDate(today.getDate() - 3);
			const oldDate = new Date(today);
			oldDate.setDate(today.getDate() - 10);

			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [
						{ date: oldDate.toISOString().slice(0, 10), tokensByModel: { 'opus': 100 } },
						{ date: recentDate.toISOString().slice(0, 10), tokensByModel: { 'sonnet': 200 } }
					],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();
			usageStore.setDateRange('7d');

			expect(usageStore.filteredDailyTokens).toHaveLength(1);
		});

		it('should return empty when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.filteredDailyTokens).toEqual([]);
		});
	});

	describe('allModels', () => {
		it('should extract and sort unique model names', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [
						{ date: '2025-01-01', tokensByModel: { 'sonnet': 100, 'opus': 200 } },
						{ date: '2025-01-02', tokensByModel: { 'opus': 300, 'haiku': 50 } }
					],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.allModels).toEqual(['haiku', 'opus', 'sonnet']);
		});

		it('should return empty array when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.allModels).toEqual([]);
		});
	});

	describe('totalToolCalls', () => {
		it('should sum tool calls across all daily activity', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [
						{ date: '2025-01-01', messageCount: 1, sessionCount: 1, toolCallCount: 5 },
						{ date: '2025-01-02', messageCount: 2, sessionCount: 1, toolCallCount: 10 },
						{ date: '2025-01-03', messageCount: 3, sessionCount: 2, toolCallCount: 15 }
					],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.totalToolCalls).toBe(30);
		});

		it('should return 0 when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.totalToolCalls).toBe(0);
		});
	});

	describe('totalCostUSD', () => {
		it('should use costUSD from stats when available', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {
						'opus': {
							inputTokens: 1000,
							outputTokens: 500,
							cacheReadInputTokens: 0,
							cacheCreationInputTokens: 0,
							webSearchRequests: 0,
							costUSD: 5.50,
							contextWindow: 200000,
							maxOutputTokens: 4096
						}
					},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.totalCostUSD).toBe(5.50);
		});

		it('should estimate cost when costUSD is 0', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {
						'opus': {
							inputTokens: 1000,
							outputTokens: 500,
							cacheReadInputTokens: 0,
							cacheCreationInputTokens: 0,
							webSearchRequests: 0,
							costUSD: 0,
							contextWindow: 200000,
							maxOutputTokens: 4096
						}
					},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			// estimateModelCost mock returns (input + output) * 0.001
			expect(usageStore.totalCostUSD).toBe(1.5);
		});

		it('should sum costs across multiple models', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {
						'opus': {
							inputTokens: 1000, outputTokens: 500,
							cacheReadInputTokens: 0, cacheCreationInputTokens: 0,
							webSearchRequests: 0, costUSD: 3.00,
							contextWindow: 200000, maxOutputTokens: 4096
						},
						'sonnet': {
							inputTokens: 2000, outputTokens: 1000,
							cacheReadInputTokens: 0, cacheCreationInputTokens: 0,
							webSearchRequests: 0, costUSD: 2.00,
							contextWindow: 200000, maxOutputTokens: 4096
						}
					},
					hourCounts: {}
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.totalCostUSD).toBe(5.00);
		});

		it('should return 0 when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			expect(usageStore.totalCostUSD).toBe(0);
		});
	});

	describe('hourCountsArray', () => {
		it('should convert hour counts to 24-element array', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: { '0': 5, '9': 20, '14': 15, '23': 8 }
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.hourCountsArray).toHaveLength(24);
			expect(usageStore.hourCountsArray[0]).toBe(5);
			expect(usageStore.hourCountsArray[9]).toBe(20);
			expect(usageStore.hourCountsArray[14]).toBe(15);
			expect(usageStore.hourCountsArray[23]).toBe(8);
			expect(usageStore.hourCountsArray[1]).toBe(0);
		});

		it('should ignore invalid hour values', async () => {
			const mockData = {
				filePath: '/path',
				exists: true,
				stats: {
					dailyActivity: [],
					dailyModelTokens: [],
					modelUsage: {},
					hourCounts: { '-1': 5, '24': 10, '25': 3 }
				}
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockData);

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			await usageStore.load();

			expect(usageStore.hourCountsArray).toHaveLength(24);
			// All should be 0 since -1, 24, 25 are out of range
			expect(usageStore.hourCountsArray.every((v) => v === 0)).toBe(true);
		});

		it('should return zeros when no stats', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			const arr = usageStore.hourCountsArray;
			expect(arr).toHaveLength(24);
			expect(arr.every((v) => v === 0)).toBe(true);
		});
	});

	describe('startPolling / stopPolling', () => {
		it('should start polling and call load periodically', async () => {
			vi.mocked(invoke).mockResolvedValue({
				filePath: '/path', exists: false, stats: null
			});

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			usageStore.startPolling(5000);

			// Advance time to trigger polling
			await vi.advanceTimersByTimeAsync(5000);
			expect(invoke).toHaveBeenCalledWith('get_usage_stats');

			await vi.advanceTimersByTimeAsync(5000);
			expect(invoke).toHaveBeenCalledTimes(2);

			usageStore.stopPolling();
		});

		it('should stop polling', async () => {
			vi.mocked(invoke).mockResolvedValue({
				filePath: '/path', exists: false, stats: null
			});

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			usageStore.startPolling(5000);

			await vi.advanceTimersByTimeAsync(5000);
			const callCount = vi.mocked(invoke).mock.calls.length;

			usageStore.stopPolling();

			await vi.advanceTimersByTimeAsync(10000);
			expect(invoke).toHaveBeenCalledTimes(callCount);
		});

		it('should stop previous polling when starting new one', async () => {
			vi.mocked(invoke).mockResolvedValue({
				filePath: '/path', exists: false, stats: null
			});

			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			usageStore.startPolling(5000);
			usageStore.startPolling(10000);

			// After 5s, the original 5s interval should not fire
			await vi.advanceTimersByTimeAsync(5000);
			expect(invoke).not.toHaveBeenCalled();

			// After 10s total, the new 10s interval should fire
			await vi.advanceTimersByTimeAsync(5000);
			expect(invoke).toHaveBeenCalledTimes(1);

			usageStore.stopPolling();
		});

		it('stopPolling should be safe to call when not polling', async () => {
			const { usageStore } = await import('$lib/stores/usageStore.svelte');
			// Should not throw
			usageStore.stopPolling();
		});
	});
});
