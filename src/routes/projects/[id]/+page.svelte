<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import { projectsStore, claudeSettingsLibrary, mcpLibrary, hookLibrary } from '$lib/stores';
	import { ProjectDashboard } from '$lib/components/projects';
	import { ArrowLeft, FolderOpen } from 'lucide-svelte';

	let isReady = $state(false);
	let loadError = $state<string | null>(null);

	let projectId = $derived(Number($page.params.id));
	let project = $derived(projectsStore.getProjectById(projectId));

	onMount(async () => {
		try {
			// Ensure stores are loaded (handles direct navigation / page refresh)
			if (projectsStore.projects.length === 0) {
				await projectsStore.loadProjects();
			}

			// Load MCP and hook libraries if not loaded
			if (mcpLibrary.mcps.length === 0) {
				await mcpLibrary.load();
			}
			if (hookLibrary.hooks.length === 0) {
				await hookLibrary.load();
			}

			// Check if project exists after loading
			const p = projectsStore.getProjectById(projectId);
			if (!p) {
				loadError = `Project with ID ${projectId} not found`;
				return;
			}

			// Set up claude settings for this project
			claudeSettingsLibrary.setProjectPath(p.path);
			claudeSettingsLibrary.setScope('project');
			await claudeSettingsLibrary.load();

			isReady = true;
		} catch (err) {
			loadError = String(err);
			console.error('Failed to initialize project dashboard:', err);
		}
	});

	onDestroy(() => {
		// Reset settings store to avoid leaking project context
		claudeSettingsLibrary.setProjectPath(null);
		claudeSettingsLibrary.setScope('user');
	});
</script>

{#if loadError}
	<div class="flex flex-col items-center justify-center h-full gap-4">
		<div class="w-16 h-16 rounded-2xl bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
			<FolderOpen class="w-8 h-8 text-red-400" />
		</div>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Project Not Found</h2>
		<p class="text-gray-500 dark:text-gray-400 text-center max-w-md">{loadError}</p>
		<button onclick={() => goto('/projects')} class="btn btn-primary mt-2">
			<ArrowLeft class="w-4 h-4 mr-2" />
			Back to Projects
		</button>
	</div>
{:else if !isReady || !project}
	<div class="flex items-center justify-center h-full">
		<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"></div>
	</div>
{:else}
	<ProjectDashboard {project} />
{/if}
