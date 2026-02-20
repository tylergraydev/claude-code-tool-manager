import type { ProjectSummary } from '$lib/types';
import { estimateSessionCost } from '$lib/types/usage';
import { sessionStore } from './sessionStore.svelte';

export const PROJECT_COLORS = ['#3b82f6', '#8b5cf6', '#f59e0b', '#ef4444', '#10b981'];

export interface ProjectComparisonData {
	project: ProjectSummary;
	color: string;
	totalTokens: number;
	estimatedCost: number;
	topTools: [string, number][];
	dateRange: string;
}

class ComparisonStoreState {
	selectedFolders = $state<Set<string>>(new Set());

	selectedProjects = $derived.by((): ProjectSummary[] => {
		return sessionStore.projects.filter((p) => this.selectedFolders.has(p.folderName));
	});

	comparisonData = $derived.by((): ProjectComparisonData[] => {
		const folders = [...this.selectedFolders];
		return this.selectedProjects.map((project) => {
			const colorIndex = folders.indexOf(project.folderName);
			const totalTokens = project.totalInputTokens + project.totalOutputTokens;
			const estimatedCost = estimateSessionCost(
				project.modelsUsed,
				project.totalInputTokens,
				project.totalOutputTokens,
				project.totalCacheReadTokens,
				project.totalCacheCreationTokens
			);
			const topTools = Object.entries(project.toolUsage)
				.sort((a, b) => b[1] - a[1]);
			const dateRange = formatDateRange(project.earliestSession, project.latestSession);

			return {
				project,
				color: PROJECT_COLORS[colorIndex] ?? '#6b7280',
				totalTokens,
				estimatedCost,
				topTools,
				dateRange
			};
		});
	});

	allTools = $derived.by((): string[] => {
		const totals: Record<string, number> = {};
		for (const d of this.comparisonData) {
			for (const [tool, count] of d.topTools) {
				totals[tool] = (totals[tool] || 0) + count;
			}
		}
		return Object.entries(totals)
			.sort((a, b) => b[1] - a[1])
			.slice(0, 10)
			.map(([tool]) => tool);
	});

	allModels = $derived.by((): string[] => {
		const modelSet = new Set<string>();
		for (const d of this.comparisonData) {
			for (const m of d.project.modelsUsed) {
				modelSet.add(m);
			}
		}
		return [...modelSet].sort();
	});

	toggleProject(folder: string) {
		const next = new Set(this.selectedFolders);
		if (next.has(folder)) {
			next.delete(folder);
		} else if (next.size < 5) {
			next.add(folder);
		}
		this.selectedFolders = next;
	}

	clearSelection() {
		this.selectedFolders = new Set();
	}
}

function formatDateRange(earliest: string | null, latest: string | null): string {
	const fmt = (iso: string | null) => {
		if (!iso) return '?';
		try {
			return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
		} catch {
			return iso;
		}
	};
	return `${fmt(earliest)} – ${fmt(latest)}`;
}

export const comparisonStore = new ComparisonStoreState();
