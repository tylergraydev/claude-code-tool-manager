<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { CommandLibrary, CommandForm } from '$lib/components/commands';
	import { ConfirmDialog } from '$lib/components/shared';
	import { commandLibrary, notifications } from '$lib/stores';
	import type { Command } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddCommand = $state(false);
	let editingCommand = $state<Command | null>(null);
	let deletingCommand = $state<Command | null>(null);

	async function handleCreateCommand(values: any) {
		try {
			await commandLibrary.create(values);
			showAddCommand = false;
			notifications.success('Command created successfully');
		} catch (err) {
			notifications.error('Failed to create command');
		}
	}

	async function handleUpdateCommand(values: any) {
		if (!editingCommand) return;
		try {
			await commandLibrary.update(editingCommand.id, values);
			editingCommand = null;
			notifications.success('Command updated successfully');
		} catch (err) {
			notifications.error('Failed to update command');
		}
	}

	async function handleDeleteCommand() {
		if (!deletingCommand) return;
		try {
			await commandLibrary.delete(deletingCommand.id);
			notifications.success('Command deleted');
		} catch (err) {
			notifications.error('Failed to delete command');
		} finally {
			deletingCommand = null;
		}
	}
</script>

<Header
	title="Commands"
	subtitle="Slash commands you invoke with /name"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddCommand = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add Command
		</button>
	</div>

	<CommandLibrary
		onEdit={(command) => (editingCommand = command)}
		onDelete={(command) => (deletingCommand = command)}
	/>
</div>

<!-- Add Command Modal -->
{#if showAddCommand}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New Command</h2>
				<CommandForm onSubmit={handleCreateCommand} onCancel={() => (showAddCommand = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit Command Modal -->
{#if editingCommand}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit Command</h2>
				<CommandForm
					initialValues={editingCommand}
					onSubmit={handleUpdateCommand}
					onCancel={() => (editingCommand = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingCommand}
	title="Delete Command"
	message="Are you sure you want to delete '/{deletingCommand?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteCommand}
	onCancel={() => (deletingCommand = null)}
/>
