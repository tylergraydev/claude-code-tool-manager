<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { SkillLibrary, SkillForm, SkillFilesEditor } from '$lib/components/skills';
	import { ConfirmDialog } from '$lib/components/shared';
	import { skillLibrary, notifications } from '$lib/stores';
	import type { Skill } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let editTab = $state<'details' | 'files'>('details');

	let showAddSkill = $state(false);
	let editingSkill = $state<Skill | null>(null);
	let deletingSkill = $state<Skill | null>(null);

	async function handleCreateSkill(values: any) {
		try {
			await skillLibrary.create(values);
			showAddSkill = false;
			notifications.success('Skill created successfully');
		} catch (err) {
			notifications.error('Failed to create skill');
		}
	}

	async function handleUpdateSkill(values: any) {
		if (!editingSkill) return;
		try {
			await skillLibrary.update(editingSkill.id, values);
			editingSkill = null;
			notifications.success('Skill updated successfully');
		} catch (err) {
			notifications.error('Failed to update skill');
		}
	}

	async function handleDeleteSkill() {
		if (!deletingSkill) return;
		try {
			await skillLibrary.delete(deletingSkill.id);
			notifications.success('Skill deleted');
		} catch (err) {
			notifications.error('Failed to delete skill');
		} finally {
			deletingSkill = null;
		}
	}
</script>

<Header
	title="Skills Library"
	subtitle="Custom slash commands - drag them to projects or global settings"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddSkill = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add Skill
		</button>
	</div>

	<SkillLibrary
		onEdit={(skill) => (editingSkill = skill)}
		onDelete={(skill) => (deletingSkill = skill)}
	/>
</div>

<!-- Add Skill Modal -->
{#if showAddSkill}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New Skill</h2>
				<SkillForm onSubmit={handleCreateSkill} onCancel={() => (showAddSkill = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit Skill Modal -->
{#if editingSkill}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Edit Skill</h2>

				<!-- Tabs for Agent Skills -->
				{#if editingSkill.skillType === 'skill'}
					<div class="flex border-b border-gray-200 dark:border-gray-700 mb-6">
						<button
							type="button"
							onclick={() => editTab = 'details'}
							class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {editTab === 'details' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'}"
						>
							Details
						</button>
						<button
							type="button"
							onclick={() => editTab = 'files'}
							class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {editTab === 'files' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'}"
						>
							Files
						</button>
					</div>
				{/if}

				{#if editTab === 'details' || editingSkill.skillType !== 'skill'}
					<SkillForm
						initialValues={editingSkill}
						onSubmit={handleUpdateSkill}
						onCancel={() => { editingSkill = null; editTab = 'details'; }}
					/>
				{:else}
					<SkillFilesEditor skillId={editingSkill.id} skillName={editingSkill.name} />
					<div class="flex justify-end mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
						<button
							type="button"
							onclick={() => { editingSkill = null; editTab = 'details'; }}
							class="btn btn-secondary"
						>
							Close
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingSkill}
	title="Delete Skill"
	message="Are you sure you want to delete '/{deletingSkill?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteSkill}
	onCancel={() => (deletingSkill = null)}
/>
