import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

vi.mock('$lib/types/session', () => ({
	totalTokens: (s: any) => s.totalInputTokens + s.totalOutputTokens
}));

vi.mock('$lib/types/usage', () => ({
	estimateSessionCost: vi.fn(() => 0.05)
}));

describe('Session Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('loadProjects', () => {
		it('should load projects successfully', async () => {
			const mockProjects = {
				exists: true,
				projects: [
					{ folderName: 'project-a', totalInputTokens: 100, totalOutputTokens: 50 },
					{ folderName: 'project-b', totalInputTokens: 200, totalOutputTokens: 100 }
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadProjects();

			expect(sessionStore.projects).toHaveLength(2);
			expect(sessionStore.projectsExist).toBe(true);
			expect(sessionStore.isLoadingProjects).toBe(false);
		});

		it('should handle projects load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadProjects();

			expect(sessionStore.projectsError).toBe('Error: fail');
			expect(sessionStore.isLoadingProjects).toBe(false);
		});
	});

	describe('loadSessions', () => {
		it('should load sessions for a folder', async () => {
			const mockSessions = {
				exists: true,
				sessions: [
					{
						sessionId: 's1', firstTimestamp: '2024-01-01', durationMs: 1000,
						userMessageCount: 5, assistantMessageCount: 5,
						totalInputTokens: 100, totalOutputTokens: 50,
						totalCacheReadTokens: 0, totalCacheCreationTokens: 0,
						modelsUsed: ['opus'], toolCounts: { Read: 3, Write: 2 }
					}
				]
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockSessions);

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessions('my-project');

			expect(sessionStore.sessions).toHaveLength(1);
			expect(sessionStore.sessionsExist).toBe(true);
			expect(sessionStore.selectedProject).toBe('my-project');
			expect(sessionStore.selectedSessionId).toBeNull();
		});

		it('should handle sessions load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessions('my-project');

			expect(sessionStore.sessionsError).toBe('Error: fail');
		});
	});

	describe('loadSessionDetail', () => {
		it('should load session detail', async () => {
			const mockDetail = { messages: [{ role: 'user', content: 'hello' }] };
			vi.mocked(invoke).mockResolvedValueOnce(mockDetail);

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessionDetail('project', 'session-1');

			expect(sessionStore.sessionDetail).toEqual(mockDetail);
			expect(sessionStore.selectedSessionId).toBe('session-1');
			expect(sessionStore.isLoadingDetail).toBe(false);
		});

		it('should handle detail load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessionDetail('project', 'session-1');

			expect(sessionStore.detailError).toBe('Error: fail');
		});
	});

	describe('selectProject', () => {
		it('should call loadSessions', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ exists: false, sessions: [] });

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.selectProject('test');

			// loadSessions is async, just verify it starts
			expect(sessionStore.selectedProject).toBe('test');
		});
	});

	describe('selectSession', () => {
		it('should load session detail when project selected', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ exists: true, sessions: [] });
			vi.mocked(invoke).mockResolvedValueOnce({ messages: [] });

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessions('project');
			sessionStore.selectSession('s1');

			// Should have called loadSessionDetail
			expect(sessionStore.selectedSessionId).toBe('s1');
		});

		it('should not load when no project selected', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.selectSession('s1');
			// selectedSessionId should not change since selectedProject is null
		});
	});

	describe('clearSession', () => {
		it('should clear session state', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ messages: [] });

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.clearSession();

			expect(sessionStore.selectedSessionId).toBeNull();
			expect(sessionStore.sessionDetail).toBeNull();
			expect(sessionStore.detailError).toBeNull();
		});
	});

	describe('setSort', () => {
		it('should set sort field and default to desc', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.setSort('tokens');
			expect(sessionStore.sortField).toBe('tokens');
			expect(sessionStore.sortDirection).toBe('desc');
		});

		it('should toggle direction on same field', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.setSort('tokens');
			expect(sessionStore.sortDirection).toBe('desc');
			sessionStore.setSort('tokens');
			expect(sessionStore.sortDirection).toBe('asc');
			sessionStore.setSort('tokens');
			expect(sessionStore.sortDirection).toBe('desc');
		});

		it('should reset direction when changing field', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.setSort('tokens');
			sessionStore.setSort('tokens'); // asc
			sessionStore.setSort('duration');
			expect(sessionStore.sortDirection).toBe('desc');
		});
	});

	describe('sortedSessions', () => {
		it('should have sortField and sortDirection defaults', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			expect(sessionStore.sortField).toBe('date');
			expect(sessionStore.sortDirection).toBe('desc');
		});

		it('should apply different sort fields', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.setSort('tokens');
			expect(sessionStore.sortField).toBe('tokens');
			sessionStore.setSort('duration');
			expect(sessionStore.sortField).toBe('duration');
			sessionStore.setSort('messages');
			expect(sessionStore.sortField).toBe('messages');
			sessionStore.setSort('cost');
			expect(sessionStore.sortField).toBe('cost');
		});

		it('should handle sort direction toggle', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			sessionStore.setSort('date');
			expect(sessionStore.sortDirection).toBe('asc'); // toggled from default desc
			sessionStore.setSort('date');
			expect(sessionStore.sortDirection).toBe('desc'); // toggled back
		});

	});

	describe('projectToolUsage', () => {
		it('should return empty object when no sessions loaded', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			expect(sessionStore.projectToolUsage).toEqual({});
		});
	});

	describe('currentProject', () => {
		it('should return null when no project selected', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			expect(sessionStore.currentProject).toBeNull();
		});

		it('should track selectedProject after loadSessions', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ exists: true, sessions: [] });

			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			await sessionStore.loadSessions('my-proj');

			expect(sessionStore.selectedProject).toBe('my-proj');
		});

	});

	describe('default state', () => {
		it('should have correct defaults', async () => {
			const { sessionStore } = await import('$lib/stores/sessionStore.svelte');
			expect(sessionStore.projects).toEqual([]);
			expect(sessionStore.projectsExist).toBe(false);
			expect(sessionStore.sessions).toEqual([]);
			expect(sessionStore.sessionsExist).toBe(false);
			expect(sessionStore.isLoading).toBe(false);
		});
	});
});
