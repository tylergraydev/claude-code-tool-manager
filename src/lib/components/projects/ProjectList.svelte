<script lang="ts">
	import type { Project } from '$lib/types';
	import { projectsStore } from '$lib/stores';
	import ProjectCard from './ProjectCard.svelte';
	import { FolderOpen, Plus } from 'lucide-svelte';

	type Props = {
		onAddProject?: () => void;
		onRemoveProject?: (project: Project) => void;
	};

	let { onAddProject, onRemoveProject }: Props = $props();
</script>

<div class="space-y-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Projects</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400">
				Drag MCPs from the library onto projects to assign them
			</p>
		</div>
		{#if onAddProject}
			<button onclick={onAddProject} class="btn btn-primary">
				<Plus class="w-4 h-4 mr-2" />
				Add Project
			</button>
		{/if}
	</div>

	<!-- Project List -->
	{#if projectsStore.isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if projectsStore.projects.length === 0}
		<div class="text-center py-12 card">
			<FolderOpen class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
			<h3 class="text-lg font-medium text-gray-900 dark:text-white">No projects added</h3>
			<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
				Add a project folder to start managing MCPs
			</p>
			{#if onAddProject}
				<button onclick={onAddProject} class="btn btn-primary">
					<Plus class="w-4 h-4 mr-2" />
					Add Your First Project
				</button>
			{/if}
		</div>
	{:else}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
			{#each projectsStore.projects as project (project.id)}
				<ProjectCard {project} onRemove={onRemoveProject} />
			{/each}
		</div>
	{/if}
</div>
