<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { RuleLibrary, RuleForm } from '$lib/components/rules';
	import { ConfirmDialog } from '$lib/components/shared';
	import { ruleLibrary, notifications } from '$lib/stores';
	import type { Rule } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddRule = $state(false);
	let editingRule = $state<Rule | null>(null);
	let deletingRule = $state<Rule | null>(null);

	async function handleCreateRule(values: any) {
		try {
			await ruleLibrary.create(values);
			showAddRule = false;
			notifications.success('Rule created successfully');
		} catch (err) {
			notifications.error('Failed to create rule');
		}
	}

	async function handleUpdateRule(values: any) {
		if (!editingRule) return;
		try {
			await ruleLibrary.update(editingRule.id, values);
			editingRule = null;
			notifications.success('Rule updated successfully');
		} catch (err) {
			notifications.error('Failed to update rule');
		}
	}

	async function handleDeleteRule() {
		if (!deletingRule) return;
		try {
			await ruleLibrary.delete(deletingRule.id);
			notifications.success('Rule deleted');
		} catch (err) {
			notifications.error('Failed to delete rule');
		} finally {
			deletingRule = null;
		}
	}
</script>

<Header
	title="Rules"
	subtitle="Conditional instruction files loaded based on file context"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddRule = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add Rule
		</button>
	</div>

	<RuleLibrary
		onEdit={(rule) => (editingRule = rule)}
		onDelete={(rule) => (deletingRule = rule)}
	/>
</div>

<!-- Add Rule Modal -->
{#if showAddRule}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New Rule</h2>
				<RuleForm onSubmit={handleCreateRule} onCancel={() => (showAddRule = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit Rule Modal -->
{#if editingRule}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">Edit Rule</h2>
				<RuleForm
					initialValues={editingRule}
					onSubmit={handleUpdateRule}
					onCancel={() => (editingRule = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingRule}
	title="Delete Rule"
	message="Are you sure you want to delete '{deletingRule?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteRule}
	onCancel={() => (deletingRule = null)}
/>
