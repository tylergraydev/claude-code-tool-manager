<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { SkillLibrary, SkillForm, SkillFilesEditor } from '$lib/components/skills';
	import { ConfirmDialog } from '$lib/components/shared';
	import { skillLibrary, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
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
			notifications.success(i18n.t('skill.created'));
		} catch (err) {
			notifications.error(i18n.t('skill.createFailed'));
		}
	}

	async function handleUpdateSkill(values: any) {
		if (!editingSkill) return;
		try {
			await skillLibrary.update(editingSkill.id, values);
			editingSkill = null;
			notifications.success(i18n.t('skill.updated'));
		} catch (err) {
			notifications.error(i18n.t('skill.updateFailed'));
		}
	}

	async function handleDeleteSkill() {
		if (!deletingSkill) return;
		try {
			await skillLibrary.delete(deletingSkill.id);
			notifications.success(i18n.t('skill.deleted'));
		} catch (err) {
			notifications.error(i18n.t('skill.deleteFailed'));
		} finally {
			deletingSkill = null;
		}
	}
</script>

<Header
	title={i18n.t('page.skills.title')}
	subtitle={i18n.t('page.skills.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddSkill = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			{i18n.t('skill.addSkill')}
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('skill.addNew')}</h2>
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">{i18n.t('skill.editSkill')}</h2>

				<!-- Tabs for editing -->
				<div class="flex border-b border-gray-200 dark:border-gray-700 mb-6">
					<button
						type="button"
						onclick={() => editTab = 'details'}
						class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {editTab === 'details' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'}"
					>
						{i18n.t('skill.details')}
					</button>
					<button
						type="button"
						onclick={() => editTab = 'files'}
						class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {editTab === 'files' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'}"
					>
						{i18n.t('skill.files')}
					</button>
				</div>

				{#if editTab === 'details'}
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
							{i18n.t('common.close')}
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingSkill}
	title={i18n.t('skill.deleteSkill')}
	message={i18n.t('skill.deleteConfirm', { name: deletingSkill?.name ?? '' })}
	confirmText={i18n.t('common.delete')}
	onConfirm={handleDeleteSkill}
	onCancel={() => (deletingSkill = null)}
/>
