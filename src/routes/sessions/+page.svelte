<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import ProjectOverviewCards from '$lib/components/sessions/ProjectOverviewCards.svelte';
	import ProjectSelector from '$lib/components/sessions/ProjectSelector.svelte';
	import SessionListTable from '$lib/components/sessions/SessionListTable.svelte';
	import SessionDetailPanel from '$lib/components/sessions/SessionDetailPanel.svelte';
	import ToolUsageChart from '$lib/components/sessions/ToolUsageChart.svelte';
	import { sessionStore } from '$lib/stores';
	import { RefreshCw, FolderSearch, FileQuestion } from 'lucide-svelte';

	onMount(() => {
		sessionStore.loadProjects();
	});

	function handleRefresh() {
		sessionStore.loadProjects();
		if (sessionStore.selectedProject) {
			sessionStore.loadSessions(sessionStore.selectedProject);
		}
	}
</script>

<Header title="Session Explorer" subtitle="Browse individual Claude Code sessions per project">
	{#snippet children()}
		<button onclick={handleRefresh} class="btn btn-ghost" title="Refresh data">
			<RefreshCw class="w-4 h-4" />
		</button>
	{/snippet}
</Header>

<div class="flex-1 overflow-auto p-6 space-y-6">
	{#if sessionStore.isLoadingProjects}
		<div class="flex items-center justify-center py-20">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if sessionStore.projectsError}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{sessionStore.projectsError}
		</div>
	{:else if !sessionStore.projectsExist || sessionStore.projects.length === 0}
		<div
			class="text-center py-16 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<div class="text-gray-400 dark:text-gray-500 mb-4">
				<FileQuestion class="w-12 h-12 mx-auto mb-3 opacity-50" />
				<p class="text-lg font-medium">No session data found</p>
				<p class="text-sm mt-1">
					Claude Code stores sessions in <code
						class="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded"
						>~/.claude/projects/</code
					>
				</p>
				<p class="text-sm mt-1">Use Claude Code to create sessions, then refresh this page.</p>
			</div>
		</div>
	{:else}
		<!-- Overview Cards -->
		<ProjectOverviewCards projects={sessionStore.projects} />

		<!-- Project Selector + Tool Usage side by side on large screens -->
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-stretch">
			<ProjectSelector
				projects={sessionStore.projects}
				selectedFolder={sessionStore.selectedProject}
				onSelect={(folder) => sessionStore.selectProject(folder)}
			/>

			{#if sessionStore.selectedProject && Object.keys(sessionStore.projectToolUsage).length > 0}
				<ToolUsageChart toolUsage={sessionStore.projectToolUsage} />
			{:else}
				<div
					class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 flex items-center justify-center h-full"
				>
					<div class="text-center text-gray-400 dark:text-gray-500">
						<FolderSearch class="w-8 h-8 mx-auto mb-2 opacity-50" />
						<p class="text-sm">Select a project to see tool usage</p>
					</div>
				</div>
			{/if}
		</div>

		<!-- Session List -->
		{#if sessionStore.isLoadingSessions}
			<div class="flex items-center justify-center py-12">
				<div
					class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
				></div>
			</div>
		{:else if sessionStore.sessionsError}
			<div
				class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
			>
				{sessionStore.sessionsError}
			</div>
		{:else if sessionStore.selectedProject && sessionStore.sessions.length > 0}
			<SessionListTable
				sessions={sessionStore.sortedSessions}
				selectedSessionId={sessionStore.selectedSessionId}
				sortField={sessionStore.sortField}
				sortDirection={sessionStore.sortDirection}
				onSelectSession={(id) => sessionStore.selectSession(id)}
				onSort={(field) => sessionStore.setSort(field)}
			/>
		{:else if sessionStore.selectedProject}
			<div
				class="text-center py-8 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
			>
				<p class="text-sm text-gray-400 dark:text-gray-500">
					No sessions found in this project
				</p>
			</div>
		{/if}

		<!-- Session Detail -->
		{#if sessionStore.isLoadingDetail}
			<div class="flex items-center justify-center py-12">
				<div
					class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
				></div>
			</div>
		{:else if sessionStore.detailError}
			<div
				class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
			>
				{sessionStore.detailError}
			</div>
		{:else if sessionStore.sessionDetail}
			<SessionDetailPanel
				detail={sessionStore.sessionDetail}
				onClose={() => sessionStore.clearSession()}
			/>
		{/if}
	{/if}
</div>
