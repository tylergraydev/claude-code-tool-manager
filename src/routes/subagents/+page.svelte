<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { SubAgentLibrary, SubAgentForm } from '$lib/components/subagents';
	import { ConfirmDialog } from '$lib/components/shared';
	import { subagentLibrary, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
	import type { SubAgent } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddSubAgent = $state(false);
	let editingSubAgent = $state<SubAgent | null>(null);
	let deletingSubAgent = $state<SubAgent | null>(null);

	async function handleCreateSubAgent(values: any) {
		try {
			await subagentLibrary.create(values);
			showAddSubAgent = false;
			notifications.success(i18n.t('subagent.created'));
		} catch (err) {
			notifications.error(i18n.t('subagent.createFailed'));
		}
	}

	async function handleUpdateSubAgent(values: any) {
		if (!editingSubAgent) return;
		try {
			await subagentLibrary.update(editingSubAgent.id, values);
			editingSubAgent = null;
			notifications.success(i18n.t('subagent.updated'));
		} catch (err) {
			notifications.error(i18n.t('subagent.updateFailed'));
		}
	}

	async function handleDeleteSubAgent() {
		if (!deletingSubAgent) return;
		try {
			await subagentLibrary.delete(deletingSubAgent.id);
			notifications.success(i18n.t('subagent.deleted'));
		} catch (err) {
			notifications.error(i18n.t('subagent.deleteFailed'));
		} finally {
			deletingSubAgent = null;
		}
	}
</script>

<Header
	title={i18n.t('page.subagents.title')}
	subtitle={i18n.t('page.subagents.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddSubAgent = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			{i18n.t('subagent.addAgent')}
		</button>
	</div>

	<SubAgentLibrary
		onEdit={(subagent) => (editingSubAgent = subagent)}
		onDelete={(subagent) => (deletingSubAgent = subagent)}
	/>
</div>

<!-- Add SubAgent Modal -->
{#if showAddSubAgent}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('subagent.addNew')}</h2>
				<SubAgentForm onSubmit={handleCreateSubAgent} onCancel={() => (showAddSubAgent = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit SubAgent Modal -->
{#if editingSubAgent}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('subagent.editAgent')}</h2>
				<SubAgentForm
					initialValues={editingSubAgent}
					onSubmit={handleUpdateSubAgent}
					onCancel={() => (editingSubAgent = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingSubAgent}
	title={i18n.t('subagent.deleteAgent')}
	message={i18n.t('subagent.deleteConfirm', { name: deletingSubAgent?.name ?? '' })}
	confirmText={i18n.t('common.delete')}
	onConfirm={handleDeleteSubAgent}
	onCancel={() => (deletingSubAgent = null)}
/>
