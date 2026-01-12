<script lang="ts">
	import type { Project } from '$lib/types';
	import { projectsStore } from '$lib/stores';
	import ProjectCard from './ProjectCard.svelte';
	import ProjectDetail from './ProjectDetail.svelte';
	import { SearchBar } from '$lib/components/shared';
	import { FolderOpen, Plus } from 'lucide-svelte';

	type Props = {
		onAddProject?: () => void;
		onRemoveProject?: (project: Project) => void;
	};

	let { onAddProject, onRemoveProject }: Props = $props();

	let selectedProject = $state<Project | null>(null);
	let searchQuery = $state('');

	// Filter projects based on search query
	let filteredProjects = $derived(
		searchQuery.trim()
			? projectsStore.sortedProjects.filter((project) => {
					const query = searchQuery.toLowerCase();
					return (
						project.name.toLowerCase().includes(query) ||
						project.path.toLowerCase().includes(query)
					);
				})
			: projectsStore.sortedProjects
	);

	function handleProjectClick(project: Project) {
		selectedProject = project;
	}

	function handleCloseDetail() {
		selectedProject = null;
		// Reload projects to get updated assignments
		projectsStore.loadProjects();
	}

	async function handleFavoriteToggle(project: Project, favorite: boolean) {
		await projectsStore.toggleFavorite(project.id, favorite);
	}
</script>

<div class="space-y-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Projects</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400">
				Click a project to manage its MCPs
			</p>
		</div>
		{#if onAddProject}
			<button onclick={onAddProject} class="btn btn-primary">
				<Plus class="w-4 h-4 mr-2" />
				Add Project
			</button>
		{/if}
	</div>

	<!-- Search Bar -->
	{#if projectsStore.projects.length > 0}
		<div class="max-w-md">
			<SearchBar bind:value={searchQuery} placeholder="Search projects..." />
		</div>
	{/if}

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
	{:else if filteredProjects.length === 0}
		<div class="text-center py-12 card">
			<FolderOpen class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
			<h3 class="text-lg font-medium text-gray-900 dark:text-white">No projects found</h3>
			<p class="text-gray-500 dark:text-gray-400 mt-1">
				No projects match "{searchQuery}"
			</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
			{#each filteredProjects as project (project.id)}
				<ProjectCard
					{project}
					onRemove={onRemoveProject}
					onClick={() => handleProjectClick(project)}
					onFavoriteToggle={handleFavoriteToggle}
				/>
			{/each}
		</div>
	{/if}
</div>

<!-- Project Detail Modal -->
{#if selectedProject}
	<ProjectDetail project={selectedProject} onClose={handleCloseDetail} />
{/if}
