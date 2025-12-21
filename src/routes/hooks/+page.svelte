<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { HookLibrary, HookForm } from '$lib/components/hooks';
	import { ConfirmDialog } from '$lib/components/shared';
	import { hookLibrary, notifications } from '$lib/stores';
	import type { Hook, CreateHookRequest } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddHook = $state(false);
	let editingHook = $state<Hook | null>(null);
	let deletingHook = $state<Hook | null>(null);

	onMount(async () => {
		await hookLibrary.load();
		await hookLibrary.loadTemplates();
		await hookLibrary.seedTemplates();
		await hookLibrary.loadGlobalHooks();
		await hookLibrary.loadAllProjectHooks();
	});

	async function handleCreateHook(values: CreateHookRequest) {
		try {
			await hookLibrary.create(values);
			showAddHook = false;
			notifications.success('Hook created successfully');
		} catch (err) {
			notifications.error('Failed to create hook');
		}
	}

	async function handleUpdateHook(values: CreateHookRequest) {
		if (!editingHook) return;
		try {
			await hookLibrary.update(editingHook.id, values);
			editingHook = null;
			notifications.success('Hook updated successfully');
		} catch (err) {
			notifications.error('Failed to update hook');
		}
	}

	async function handleDeleteHook() {
		if (!deletingHook) return;
		try {
			await hookLibrary.delete(deletingHook.id);
			notifications.success('Hook deleted');
		} catch (err) {
			notifications.error('Failed to delete hook');
		} finally {
			deletingHook = null;
		}
	}

	async function handleDuplicate(hook: Hook) {
		try {
			const newName = `${hook.name}-copy`;
			await hookLibrary.create({
				name: newName,
				description: hook.description,
				eventType: hook.eventType,
				matcher: hook.matcher,
				hookType: hook.hookType,
				command: hook.command,
				prompt: hook.prompt,
				timeout: hook.timeout,
				tags: hook.tags
			});
			notifications.success('Hook duplicated');
		} catch (err) {
			notifications.error('Failed to duplicate hook');
		}
	}
</script>

<Header
	title="Hooks Library"
	subtitle="Event-driven automations - run commands or inject prompts on Claude Code events"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddHook = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add Hook
		</button>
	</div>

	<HookLibrary
		onEdit={(hook) => (editingHook = hook)}
		onDelete={(hook) => (deletingHook = hook)}
		onDuplicate={handleDuplicate}
	/>
</div>

<!-- Add Hook Modal -->
{#if showAddHook}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto"
		>
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New Hook</h2>
				<HookForm
					templates={hookLibrary.templates}
					onSubmit={handleCreateHook}
					onCancel={() => (showAddHook = false)}
				/>
			</div>
		</div>
	</div>
{/if}

<!-- Edit Hook Modal -->
{#if editingHook}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto"
		>
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit Hook</h2>
				<HookForm
					initialValues={editingHook}
					templates={hookLibrary.templates}
					onSubmit={handleUpdateHook}
					onCancel={() => (editingHook = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingHook}
	title="Delete Hook"
	message="Are you sure you want to delete '{deletingHook?.name}'? This will remove it from all projects and global settings."
	confirmText="Delete"
	onConfirm={handleDeleteHook}
	onCancel={() => (deletingHook = null)}
/>
