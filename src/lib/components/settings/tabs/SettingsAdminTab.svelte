<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { ManagedSettingsViewer } from '$lib/components/admin-settings';
	import { notifications } from '$lib/stores';
	import type { ManagedSettingsInfo } from '$lib/types';
	import { RefreshCw } from 'lucide-svelte';

	let info = $state<ManagedSettingsInfo | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	async function load() {
		isLoading = true;
		error = null;
		try {
			info = await invoke<ManagedSettingsInfo>('get_managed_settings');
		} catch (err) {
			error = String(err);
		} finally {
			isLoading = false;
		}
	}

	async function handleRefresh() {
		await load();
		notifications.success('Managed settings refreshed');
	}

	onMount(() => {
		load();
	});
</script>

<div class="flex items-center justify-end mb-6">
	<button
		onclick={handleRefresh}
		class="btn btn-ghost"
		title="Refresh managed settings"
		disabled={isLoading}
	>
		<RefreshCw class="w-4 h-4" />
	</button>
</div>

{#if isLoading}
	<div class="flex items-center justify-center py-20">
		<div
			class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
		></div>
	</div>
{:else if error}
	<div
		class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
	>
		{error}
	</div>
{:else if info}
	<ManagedSettingsViewer {info} />
{/if}
