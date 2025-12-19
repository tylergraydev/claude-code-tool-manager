<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { McpLibrary, McpForm } from '$lib/components/mcp';
	import { ConfirmDialog } from '$lib/components/shared';
	import { mcpLibrary, notifications } from '$lib/stores';
	import type { Mcp } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddMcp = $state(false);
	let editingMcp = $state<Mcp | null>(null);
	let deletingMcp = $state<Mcp | null>(null);

	async function handleCreateMcp(values: any) {
		try {
			await mcpLibrary.create(values);
			showAddMcp = false;
			notifications.success('MCP created successfully');
		} catch (err) {
			notifications.error('Failed to create MCP');
		}
	}

	async function handleUpdateMcp(values: any) {
		if (!editingMcp) return;
		try {
			await mcpLibrary.update(editingMcp.id, values);
			editingMcp = null;
			notifications.success('MCP updated successfully');
		} catch (err) {
			notifications.error('Failed to update MCP');
		}
	}

	async function handleDeleteMcp() {
		if (!deletingMcp) return;
		try {
			await mcpLibrary.delete(deletingMcp.id);
			notifications.success('MCP deleted');
		} catch (err) {
			notifications.error('Failed to delete MCP');
		} finally {
			deletingMcp = null;
		}
	}

	async function handleDuplicateMcp(mcp: Mcp) {
		try {
			await mcpLibrary.duplicate(mcp.id);
			notifications.success('MCP duplicated');
		} catch (err) {
			notifications.error('Failed to duplicate MCP');
		}
	}
</script>

<Header
	title="MCP Library"
	subtitle="All your MCP servers in one place - drag them to projects or global settings"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddMcp = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add MCP
		</button>
	</div>

	<McpLibrary
		onEdit={(mcp) => (editingMcp = mcp)}
		onDelete={(mcp) => (deletingMcp = mcp)}
		onDuplicate={handleDuplicateMcp}
	/>
</div>

<!-- Add MCP Modal -->
{#if showAddMcp}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New MCP</h2>
				<McpForm onSubmit={handleCreateMcp} onCancel={() => (showAddMcp = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit MCP Modal -->
{#if editingMcp}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit MCP</h2>
				<McpForm
					initialValues={editingMcp}
					onSubmit={handleUpdateMcp}
					onCancel={() => (editingMcp = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingMcp}
	title="Delete MCP"
	message="Are you sure you want to delete '{deletingMcp?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteMcp}
	onCancel={() => (deletingMcp = null)}
/>
