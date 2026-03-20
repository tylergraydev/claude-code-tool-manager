<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import {
		StatusLineLibrary,
		StatusLineBuilder,
		StatusLineGallery,
		StatusLineForm
	} from '$lib/components/statusline';
	import { ConfirmDialog } from '$lib/components/shared';
	import { statuslineLibrary, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
	import type { StatusLine, CreateStatusLineRequest, StatusLineGalleryEntry } from '$lib/types';
	import { parseSegmentsJson } from '$lib/types';
	import { Plus, PenTool, Package } from 'lucide-svelte';

	let activeTab = $state<'library' | 'builder' | 'gallery'>('library');
	let showAddRaw = $state(false);
	let editingStatusLine = $state<StatusLine | null>(null);
	let editingInBuilder = $state<StatusLine | null>(null);
	let deletingStatusLine = $state<StatusLine | null>(null);

	onMount(async () => {
		await statuslineLibrary.load();
		await statuslineLibrary.loadGallery();
	});

	async function handleActivate(sl: StatusLine) {
		try {
			await statuslineLibrary.activate(sl.id);
			notifications.success(i18n.t('statusline.activated', { name: sl.name }));
		} catch (err) {
			notifications.error(i18n.t('statusline.activateFailed'));
		}
	}

	async function handleDeactivate() {
		try {
			await statuslineLibrary.deactivate();
			notifications.success(i18n.t('statusline.deactivated'));
		} catch (err) {
			notifications.error(i18n.t('statusline.deactivateFailed'));
		}
	}

	async function handleDelete() {
		if (!deletingStatusLine) return;
		try {
			await statuslineLibrary.delete(deletingStatusLine.id);
			notifications.success(i18n.t('statusline.deleted'));
		} catch (err) {
			notifications.error(i18n.t('statusline.deleteFailed'));
		} finally {
			deletingStatusLine = null;
		}
	}

	async function handleCreateRaw(request: CreateStatusLineRequest) {
		try {
			await statuslineLibrary.create(request);
			showAddRaw = false;
			notifications.success(i18n.t('statusline.created'));
		} catch (err) {
			notifications.error(i18n.t('statusline.createFailed'));
		}
	}

	async function handleUpdateRaw(request: CreateStatusLineRequest) {
		if (!editingStatusLine) return;
		try {
			await statuslineLibrary.update(editingStatusLine.id, request);
			editingStatusLine = null;
			notifications.success(i18n.t('statusline.updated'));
		} catch (err) {
			notifications.error(i18n.t('statusline.updateFailed'));
		}
	}

	function handleEdit(sl: StatusLine) {
		if (sl.segmentsJson) {
			editingInBuilder = sl;
			activeTab = 'builder';
		} else {
			editingStatusLine = sl;
		}
	}

	async function handleBuilderSave(request: CreateStatusLineRequest) {
		try {
			if (editingInBuilder) {
				await statuslineLibrary.update(editingInBuilder.id, request);
				notifications.success(i18n.t('statusline.updated'));
				editingInBuilder = null;
			} else {
				await statuslineLibrary.create(request);
				notifications.success(i18n.t('statusline.saved'));
			}
			activeTab = 'library';
		} catch (err) {
			notifications.error(i18n.t('statusline.saveFailed'));
		}
	}

	async function handleBuilderActivate(request: CreateStatusLineRequest) {
		try {
			let sl: StatusLine;
			if (editingInBuilder) {
				sl = await statuslineLibrary.update(editingInBuilder.id, request);
				editingInBuilder = null;
			} else {
				sl = await statuslineLibrary.create(request);
			}
			await statuslineLibrary.activate(sl.id);
			notifications.success(i18n.t('statusline.savedAndActivated', { name: sl.name }));
			activeTab = 'library';
		} catch (err) {
			notifications.error(i18n.t('statusline.saveActivateFailed'));
		}
	}

	async function handleGalleryInstall(entry: StatusLineGalleryEntry) {
		try {
			const sl = await statuslineLibrary.installPremade(entry);
			notifications.success(i18n.t('statusline.addedToLibrary', { name: sl.name }));
		} catch (err) {
			notifications.error(i18n.t('statusline.installFailed'));
		}
	}
</script>

<Header
	title={i18n.t('page.statusline.title')}
	subtitle={i18n.t('page.statusline.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<!-- Tab bar -->
	<div class="flex items-center justify-between mb-6">
		<div class="flex border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
			<button
				onclick={() => (activeTab = 'library')}
				class="px-4 py-2 text-sm font-medium transition-colors
					{activeTab === 'library'
						? 'bg-primary-600 text-white'
						: 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
			>
				{i18n.t('statusline.myStatusLines')}
			</button>
			<button
				onclick={() => (activeTab = 'builder')}
				class="px-4 py-2 text-sm font-medium transition-colors border-x border-gray-200 dark:border-gray-700
					{activeTab === 'builder'
						? 'bg-primary-600 text-white'
						: 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
			>
				<span class="flex items-center gap-1.5">
					<PenTool class="w-3.5 h-3.5" />
					{i18n.t('statusline.builder')}
				</span>
			</button>
			<button
				onclick={() => (activeTab = 'gallery')}
				class="px-4 py-2 text-sm font-medium transition-colors
					{activeTab === 'gallery'
						? 'bg-primary-600 text-white'
						: 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
			>
				<span class="flex items-center gap-1.5">
					<Package class="w-3.5 h-3.5" />
					{i18n.t('statusline.gallery')}
				</span>
			</button>
		</div>

		{#if activeTab === 'library'}
			<button onclick={() => (showAddRaw = true)} class="btn btn-primary">
				<Plus class="w-4 h-4 mr-2" />
				{i18n.t('statusline.addRawCommand')}
			</button>
		{/if}
	</div>

	<!-- Tab content -->
	{#if activeTab === 'library'}
		<StatusLineLibrary
			onActivate={handleActivate}
			onDeactivate={handleDeactivate}
			onEdit={handleEdit}
			onDelete={(sl) => (deletingStatusLine = sl)}
		/>
	{:else if activeTab === 'builder'}
		{#key editingInBuilder?.id}
			{@const parsed = editingInBuilder?.segmentsJson ? parseSegmentsJson(editingInBuilder.segmentsJson) : undefined}
			<StatusLineBuilder
				initialName={editingInBuilder?.name}
				initialDescription={editingInBuilder?.description ?? undefined}
				initialPadding={editingInBuilder?.padding}
				initialSegments={parsed?.segments}
				initialTheme={parsed?.theme}
				onSave={handleBuilderSave}
				onActivate={handleBuilderActivate}
			/>
		{/key}
	{:else if activeTab === 'gallery'}
		<StatusLineGallery onInstall={handleGalleryInstall} />
	{/if}
</div>

<!-- Add Raw Modal -->
{#if showAddRaw}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-lg w-full mx-4">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('statusline.addRaw')}</h2>
				<StatusLineForm
					onSubmit={handleCreateRaw}
					onCancel={() => (showAddRaw = false)}
				/>
			</div>
		</div>
	</div>
{/if}

<!-- Edit Modal -->
{#if editingStatusLine}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-lg w-full mx-4">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('statusline.editStatusLine')}</h2>
				<StatusLineForm
					initialValues={editingStatusLine}
					onSubmit={handleUpdateRaw}
					onCancel={() => (editingStatusLine = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingStatusLine}
	title={i18n.t('statusline.deleteStatusLine')}
	message={i18n.t('statusline.deleteConfirm', { name: deletingStatusLine?.name ?? '' })}
	confirmText={i18n.t('common.delete')}
	onConfirm={handleDelete}
	onCancel={() => (deletingStatusLine = null)}
/>
