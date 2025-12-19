<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { SubAgentLibrary, SubAgentForm } from '$lib/components/subagents';
	import { ConfirmDialog } from '$lib/components/shared';
	import { subagentLibrary, notifications } from '$lib/stores';
	import type { SubAgent } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddSubAgent = $state(false);
	let editingSubAgent = $state<SubAgent | null>(null);
	let deletingSubAgent = $state<SubAgent | null>(null);

	async function handleCreateSubAgent(values: any) {
		try {
			await subagentLibrary.create(values);
			showAddSubAgent = false;
			notifications.success('Sub-agent created successfully');
		} catch (err) {
			notifications.error('Failed to create sub-agent');
		}
	}

	async function handleUpdateSubAgent(values: any) {
		if (!editingSubAgent) return;
		try {
			await subagentLibrary.update(editingSubAgent.id, values);
			editingSubAgent = null;
			notifications.success('Sub-agent updated successfully');
		} catch (err) {
			notifications.error('Failed to update sub-agent');
		}
	}

	async function handleDeleteSubAgent() {
		if (!deletingSubAgent) return;
		try {
			await subagentLibrary.delete(deletingSubAgent.id);
			notifications.success('Sub-agent deleted');
		} catch (err) {
			notifications.error('Failed to delete sub-agent');
		} finally {
			deletingSubAgent = null;
		}
	}
</script>

<Header
	title="Sub-Agents Library"
	subtitle="Custom sub-agents - drag them to projects or global settings"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddSubAgent = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add Sub-Agent
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New Sub-Agent</h2>
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit Sub-Agent</h2>
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
	title="Delete Sub-Agent"
	message="Are you sure you want to delete '{deletingSubAgent?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteSubAgent}
	onCancel={() => (deletingSubAgent = null)}
/>
