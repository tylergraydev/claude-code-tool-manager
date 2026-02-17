<script lang="ts">
	import { onMount } from 'svelte';
	import { KeybindingsEditor } from '$lib/components/keybindings';
	import { keybindingsLibrary, notifications } from '$lib/stores';
	import { RefreshCw } from 'lucide-svelte';

	onMount(async () => {
		await keybindingsLibrary.load();
		keybindingsLibrary.expandAll();
	});

	async function handleRefresh() {
		await keybindingsLibrary.load();
		notifications.success('Keybindings refreshed');
	}
</script>

<div class="flex items-center justify-end mb-4">
	<button onclick={handleRefresh} class="btn btn-ghost" title="Refresh from file">
		<RefreshCw class="w-4 h-4" />
	</button>
</div>

{#if keybindingsLibrary.isLoading}
	<div class="flex items-center justify-center py-20">
		<div
			class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
		></div>
	</div>
{:else if keybindingsLibrary.error}
	<div
		class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
	>
		{keybindingsLibrary.error}
	</div>
{:else}
	<KeybindingsEditor />
{/if}
