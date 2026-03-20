<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { HookLibrary, HookForm, HookExportModal, SoundHookWizard } from '$lib/components/hooks';
	import { SoundBrowser } from '$lib/components/sounds';
	import { ConfirmDialog } from '$lib/components/shared';
	import { hookLibrary, soundLibrary, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
	import type { Hook, CreateHookRequest } from '$lib/types';
	import { Plus, Volume2, Download, Music } from 'lucide-svelte';

	let showAddHook = $state(false);
	let editingHook = $state<Hook | null>(null);
	let deletingHook = $state<Hook | null>(null);
	let showSoundWizard = $state(false);
	let showExportModal = $state(false);
	let showSoundBrowser = $state(false);

	onMount(async () => {
		await hookLibrary.load();
		await hookLibrary.loadTemplates();
		await hookLibrary.seedTemplates();
		await hookLibrary.loadGlobalHooks();
		await hookLibrary.loadAllProjectHooks();
		// Pre-load sounds for the wizard
		await soundLibrary.load();
	});

	async function handleCreateHook(values: CreateHookRequest) {
		try {
			await hookLibrary.create(values);
			showAddHook = false;
			notifications.success(i18n.t('hook.created'));
		} catch (err) {
			notifications.error(i18n.t('hook.createFailed'));
		}
	}

	async function handleUpdateHook(values: CreateHookRequest) {
		if (!editingHook) return;
		try {
			await hookLibrary.update(editingHook.id, values);
			editingHook = null;
			notifications.success(i18n.t('hook.updated'));
		} catch (err) {
			notifications.error(i18n.t('hook.updateFailed'));
		}
	}

	async function handleDeleteHook() {
		if (!deletingHook) return;
		try {
			await hookLibrary.delete(deletingHook.id);
			notifications.success(i18n.t('hook.deleted'));
		} catch (err) {
			notifications.error(i18n.t('hook.deleteFailed'));
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
			notifications.success(i18n.t('hook.duplicated'));
		} catch (err) {
			notifications.error(i18n.t('hook.duplicateFailed'));
		}
	}
</script>

<Header
	title={i18n.t('page.hooks.title')}
	subtitle={i18n.t('page.hooks.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex flex-wrap gap-3 justify-end mb-6">
		<button onclick={() => (showSoundWizard = true)} class="btn btn-secondary">
			<Volume2 class="w-4 h-4 mr-2" />
			{i18n.t('hook.soundNotifications')}
		</button>
		<button onclick={() => (showExportModal = true)} class="btn btn-secondary">
			<Download class="w-4 h-4 mr-2" />
			{i18n.t('common.export')}
		</button>
		<button onclick={() => (showSoundBrowser = true)} class="btn btn-secondary">
			<Music class="w-4 h-4 mr-2" />
			{i18n.t('hook.manageSounds')}
		</button>
		<button onclick={() => (showAddHook = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			{i18n.t('hook.addHook')}
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('hook.addNew')}</h2>
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('hook.editHook')}</h2>
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
	title={i18n.t('hook.deleteHook')}
	message={i18n.t('hook.deleteConfirm', { name: deletingHook?.name ?? '' })}
	confirmText={i18n.t('common.delete')}
	onConfirm={handleDeleteHook}
	onCancel={() => (deletingHook = null)}
/>

<!-- Sound Hook Wizard -->
{#if showSoundWizard}
	<SoundHookWizard
		onClose={() => (showSoundWizard = false)}
		onComplete={async () => {
			await hookLibrary.load();
			await hookLibrary.loadGlobalHooks();
		}}
	/>
{/if}

<!-- Export Modal -->
{#if showExportModal}
	<HookExportModal onClose={() => (showExportModal = false)} />
{/if}

<!-- Sound Browser Modal -->
{#if showSoundBrowser}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden">
			<SoundBrowser onClose={() => (showSoundBrowser = false)} />
		</div>
	</div>
{/if}
