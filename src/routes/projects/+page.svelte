<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { ProjectList } from '$lib/components/projects';
	import { ConfirmDialog } from '$lib/components/shared';
	import { projectsStore, notifications } from '$lib/stores';
	import type { Project } from '$lib/types';

	let deletingProject = $state<Project | null>(null);

	async function handleAddProject() {
		try {
			const path = await projectsStore.browseForProject();
			if (path) {
				const name = path.split(/[/\\]/).pop() || 'Project';
				await projectsStore.addProject({ name, path });
				notifications.success('Project added');
			}
		} catch (err) {
			notifications.error('Failed to add project');
		}
	}

	async function handleRemoveProject() {
		if (!deletingProject) return;
		try {
			await projectsStore.removeProject(deletingProject.id);
			notifications.success('Project removed');
		} catch (err) {
			notifications.error('Failed to remove project');
		} finally {
			deletingProject = null;
		}
	}
</script>

<Header
	title="Projects"
	subtitle="Manage MCP assignments for your Claude Code projects"
/>

<div class="flex-1 overflow-auto p-6">
	<ProjectList
		onAddProject={handleAddProject}
		onRemoveProject={(project) => (deletingProject = project)}
	/>
</div>

<ConfirmDialog
	open={!!deletingProject}
	title="Remove Project"
	message="Are you sure you want to remove '{deletingProject?.name}'? This won't delete any files."
	confirmText="Remove"
	variant="warning"
	onConfirm={handleRemoveProject}
	onCancel={() => (deletingProject = null)}
/>
