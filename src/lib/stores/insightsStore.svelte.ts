import { invoke } from '@tauri-apps/api/core';
import type { InsightsReportInfo, SessionFacetsInfo, SessionFacet } from '$lib/types';

class InsightsStoreState {
	reportInfo = $state<InsightsReportInfo | null>(null);
	facetsInfo = $state<SessionFacetsInfo | null>(null);
	isLoadingReport = $state(false);
	isLoadingFacets = $state(false);
	reportError = $state<string | null>(null);
	facetsError = $state<string | null>(null);

	reportExists = $derived(this.reportInfo?.exists ?? false);
	reportHtml = $derived(this.reportInfo?.htmlContent ?? null);
	reportFilePath = $derived(this.reportInfo?.filePath ?? '');
	facetsExist = $derived(this.facetsInfo?.exists ?? false);
	facets = $derived(this.facetsInfo?.facets ?? []);

	isLoading = $derived(this.isLoadingReport || this.isLoadingFacets);

	outcomeCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const facet of this.facets) {
			if (facet.outcome) {
				counts[facet.outcome] = (counts[facet.outcome] || 0) + 1;
			}
		}
		return counts;
	});

	helpfulnessCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const facet of this.facets) {
			if (facet.claudeHelpfulness) {
				counts[facet.claudeHelpfulness] = (counts[facet.claudeHelpfulness] || 0) + 1;
			}
		}
		return counts;
	});

	aggregatedFrictionCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const facet of this.facets) {
			for (const [key, value] of Object.entries(facet.frictionCounts)) {
				counts[key] = (counts[key] || 0) + value;
			}
		}
		return counts;
	});

	async load() {
		console.log('[insightsStore] Loading insights data...');
		await Promise.all([this.loadReport(), this.loadFacets()]);
	}

	async loadReport() {
		this.isLoadingReport = true;
		this.reportError = null;
		try {
			this.reportInfo = await invoke<InsightsReportInfo>('get_insights_report');
			console.log('[insightsStore] Loaded insights report');
		} catch (e) {
			this.reportError = String(e);
			console.error('[insightsStore] Failed to load insights report:', e);
		} finally {
			this.isLoadingReport = false;
		}
	}

	async loadFacets() {
		this.isLoadingFacets = true;
		this.facetsError = null;
		try {
			this.facetsInfo = await invoke<SessionFacetsInfo>('get_session_facets');
			console.log('[insightsStore] Loaded session facets');
		} catch (e) {
			this.facetsError = String(e);
			console.error('[insightsStore] Failed to load session facets:', e);
		} finally {
			this.isLoadingFacets = false;
		}
	}
}

export const insightsStore = new InsightsStoreState();
