<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { ProjectList } from '$lib/components/projects';
	import { ConfirmDialog } from '$lib/components/shared';
	import { projectsStore, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
	import type { Project } from '$lib/types';

	let deletingProject = $state<Project | null>(null);

	async function handleAddProject() {
		try {
			const path = await projectsStore.browseForProject();
			if (path) {
				const name = path.split(/[/\\]/).pop() || 'Project';
				await projectsStore.addProject({ name, path });
				notifications.success(i18n.t('project.added'));
			}
		} catch (err) {
			notifications.error(i18n.t('project.addFailed'));
		}
	}

	async function handleRemoveProject() {
		if (!deletingProject) return;
		try {
			await projectsStore.removeProject(deletingProject.id);
			notifications.success(i18n.t('project.removed'));
		} catch (err) {
			notifications.error(i18n.t('project.removeFailed'));
		} finally {
			deletingProject = null;
		}
	}
</script>

<Header
	title={i18n.t('page.projects.title')}
	subtitle={i18n.t('page.projects.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<ProjectList
		onAddProject={handleAddProject}
		onRemoveProject={(project) => (deletingProject = project)}
	/>
</div>

<ConfirmDialog
	open={!!deletingProject}
	title={i18n.t('project.removeProject')}
	message={i18n.t('project.removeConfirm', { name: deletingProject?.name ?? '' })}
	confirmText={i18n.t('common.remove')}
	variant="warning"
	onConfirm={handleRemoveProject}
	onCancel={() => (deletingProject = null)}
/>
