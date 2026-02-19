import { invoke } from '@tauri-apps/api/core';
import type {
	ProjectListInfo,
	SessionListInfo,
	SessionDetail,
	ProjectSummary,
	SessionSummary,
	SessionSortField
} from '$lib/types';
import { totalTokens } from '$lib/types/session';
import { estimateSessionCost } from '$lib/types/usage';

class SessionStoreState {
	projectList = $state<ProjectListInfo | null>(null);
	sessionList = $state<SessionListInfo | null>(null);
	sessionDetail = $state<SessionDetail | null>(null);
	selectedProject = $state<string | null>(null);
	selectedSessionId = $state<string | null>(null);

	isLoadingProjects = $state(false);
	isLoadingSessions = $state(false);
	isLoadingDetail = $state(false);
	projectsError = $state<string | null>(null);
	sessionsError = $state<string | null>(null);
	detailError = $state<string | null>(null);

	sortField = $state<SessionSortField>('date');
	sortDirection = $state<'asc' | 'desc'>('desc');

	// Derived state
	projects = $derived(this.projectList?.projects ?? []);
	projectsExist = $derived(this.projectList?.exists ?? false);
	sessions = $derived(this.sessionList?.sessions ?? []);
	sessionsExist = $derived(this.sessionList?.exists ?? false);

	isLoading = $derived(this.isLoadingProjects || this.isLoadingSessions || this.isLoadingDetail);

	sortedSessions = $derived.by(() => {
		const items = [...this.sessions];
		const dir = this.sortDirection === 'asc' ? 1 : -1;

		items.sort((a, b) => {
			switch (this.sortField) {
				case 'date': {
					const aTs = a.firstTimestamp ?? '';
					const bTs = b.firstTimestamp ?? '';
					return aTs < bTs ? -dir : aTs > bTs ? dir : 0;
				}
				case 'tokens':
					return (totalTokens(a) - totalTokens(b)) * dir;
				case 'duration':
					return (a.durationMs - b.durationMs) * dir;
				case 'messages':
					return (a.userMessageCount + a.assistantMessageCount - (b.userMessageCount + b.assistantMessageCount)) * dir;
				case 'cost': {
					const aCost = estimateSessionCost(a.modelsUsed, a.totalInputTokens, a.totalOutputTokens, a.totalCacheReadTokens, a.totalCacheCreationTokens);
					const bCost = estimateSessionCost(b.modelsUsed, b.totalInputTokens, b.totalOutputTokens, b.totalCacheReadTokens, b.totalCacheCreationTokens);
					return (aCost - bCost) * dir;
				}
				default:
					return 0;
			}
		});

		return items;
	});

	projectToolUsage = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const session of this.sessions) {
			for (const [tool, count] of Object.entries(session.toolCounts)) {
				counts[tool] = (counts[tool] || 0) + count;
			}
		}
		return counts;
	});

	currentProject = $derived.by(() => {
		if (!this.selectedProject) return null;
		return this.projects.find((p) => p.folderName === this.selectedProject) ?? null;
	});

	async loadProjects() {
		console.log('[sessionStore] Loading session projects...');
		this.isLoadingProjects = true;
		this.projectsError = null;
		try {
			this.projectList = await invoke<ProjectListInfo>('get_session_projects');
			console.log('[sessionStore] Loaded', this.projects.length, 'projects');
		} catch (e) {
			this.projectsError = String(e);
			console.error('[sessionStore] Failed to load projects:', e);
		} finally {
			this.isLoadingProjects = false;
		}
	}

	async loadSessions(folder: string) {
		console.log('[sessionStore] Loading sessions for:', folder);
		this.isLoadingSessions = true;
		this.sessionsError = null;
		this.selectedProject = folder;
		this.selectedSessionId = null;
		this.sessionDetail = null;
		try {
			this.sessionList = await invoke<SessionListInfo>('get_project_sessions', {
				projectFolder: folder
			});
			console.log('[sessionStore] Loaded', this.sessions.length, 'sessions');
		} catch (e) {
			this.sessionsError = String(e);
			console.error('[sessionStore] Failed to load sessions:', e);
		} finally {
			this.isLoadingSessions = false;
		}
	}

	async loadSessionDetail(folder: string, id: string) {
		console.log('[sessionStore] Loading session detail:', id);
		this.isLoadingDetail = true;
		this.detailError = null;
		this.selectedSessionId = id;
		try {
			this.sessionDetail = await invoke<SessionDetail>('get_session_detail', {
				projectFolder: folder,
				sessionId: id
			});
			console.log(
				'[sessionStore] Loaded detail with',
				this.sessionDetail?.messages.length,
				'messages'
			);
		} catch (e) {
			this.detailError = String(e);
			console.error('[sessionStore] Failed to load session detail:', e);
		} finally {
			this.isLoadingDetail = false;
		}
	}

	selectProject(folder: string) {
		this.loadSessions(folder);
	}

	selectSession(id: string) {
		if (this.selectedProject) {
			this.loadSessionDetail(this.selectedProject, id);
		}
	}

	clearSession() {
		this.selectedSessionId = null;
		this.sessionDetail = null;
		this.detailError = null;
	}

	setSort(field: SessionSortField) {
		if (this.sortField === field) {
			this.sortDirection = this.sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			this.sortField = field;
			this.sortDirection = 'desc';
		}
	}
}

export const sessionStore = new SessionStoreState();
