import { invoke } from '@tauri-apps/api/core';
import type { StatsCacheInfo, DateRangeFilter } from '$lib/types';

class UsageStoreState {
	data = $state<StatsCacheInfo | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);
	dateRange = $state<DateRangeFilter>('30d');

	stats = $derived(this.data?.stats ?? null);
	exists = $derived(this.data?.exists ?? false);
	filePath = $derived(this.data?.filePath ?? '');

	filteredDailyActivity = $derived.by(() => {
		const activity = this.stats?.dailyActivity ?? [];
		return this.filterByDateRange(activity);
	});

	filteredDailyTokens = $derived.by(() => {
		const tokens = this.stats?.dailyModelTokens ?? [];
		return this.filterByDateRange(tokens);
	});

	allModels = $derived.by(() => {
		const models = new Set<string>();
		for (const entry of this.stats?.dailyModelTokens ?? []) {
			for (const model of Object.keys(entry.tokensByModel)) {
				models.add(model);
			}
		}
		return [...models].sort();
	});

	totalToolCalls = $derived.by(() => {
		const activity = this.stats?.dailyActivity ?? [];
		return activity.reduce((sum, d) => sum + d.toolCallCount, 0);
	});

	hourCountsArray = $derived.by(() => {
		const counts = this.stats?.hourCounts ?? {};
		const arr: number[] = new Array(24).fill(0);
		for (const [hour, count] of Object.entries(counts)) {
			const h = parseInt(hour, 10);
			if (h >= 0 && h < 24) arr[h] = count;
		}
		return arr;
	});

	private filterByDateRange<T extends { date: string }>(items: T[]): T[] {
		if (this.dateRange === 'all') return items;
		const days = this.dateRange === '7d' ? 7 : 30;
		const cutoff = new Date();
		cutoff.setDate(cutoff.getDate() - days);
		const cutoffStr = cutoff.toISOString().slice(0, 10);
		return items.filter((item) => item.date >= cutoffStr);
	}

	async load() {
		console.log('[usageStore] Loading usage stats...');
		this.isLoading = true;
		this.error = null;
		try {
			this.data = await invoke<StatsCacheInfo>('get_usage_stats');
			console.log('[usageStore] Loaded usage stats');
		} catch (e) {
			this.error = String(e);
			console.error('[usageStore] Failed to load usage stats:', e);
		} finally {
			this.isLoading = false;
		}
	}

	setDateRange(range: DateRangeFilter) {
		this.dateRange = range;
	}
}

export const usageStore = new UsageStoreState();
