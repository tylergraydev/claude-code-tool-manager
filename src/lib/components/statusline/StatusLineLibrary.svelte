<script lang="ts">
	import { statuslineLibrary } from '$lib/stores';
	import StatusLineCard from './StatusLineCard.svelte';
	import type { StatusLine } from '$lib/types';
	import { Search, PanelBottom } from 'lucide-svelte';

	type Props = {
		onActivate?: (sl: StatusLine) => void;
		onDeactivate?: () => void;
		onEdit?: (sl: StatusLine) => void;
		onDelete?: (sl: StatusLine) => void;
	};

	let { onActivate, onDeactivate, onEdit, onDelete }: Props = $props();
</script>

<div>
	<!-- Search -->
	<div class="relative mb-4">
		<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
		<input
			type="text"
			placeholder="Search status lines..."
			value={statuslineLibrary.searchQuery}
			oninput={(e) => statuslineLibrary.setSearch((e.target as HTMLInputElement).value)}
			class="w-full pl-10 pr-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
		/>
	</div>

	{#if statuslineLibrary.isLoading}
		<div class="text-center py-12 text-gray-500 dark:text-gray-400">Loading...</div>
	{:else if statuslineLibrary.filteredStatusLines.length === 0}
		<div class="text-center py-12">
			<PanelBottom class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-3" />
			<p class="text-gray-500 dark:text-gray-400">
				{statuslineLibrary.searchQuery ? 'No status lines match your search' : 'No status lines yet'}
			</p>
			<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
				Create a custom one, install from the gallery, or paste a raw command
			</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each statuslineLibrary.filteredStatusLines as sl (sl.id)}
				<StatusLineCard
					statusline={sl}
					{onActivate}
					{onDeactivate}
					{onEdit}
					{onDelete}
				/>
			{/each}
		</div>
	{/if}
</div>
