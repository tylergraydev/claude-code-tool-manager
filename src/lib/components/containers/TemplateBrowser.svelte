<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import type { ContainerTemplate } from '$lib/types';
	import TemplateCard from './TemplateCard.svelte';

	let { onUse }: {
		onUse: (template: ContainerTemplate) => void;
	} = $props();

	let filterCategory = $state('all');

	const categories = $derived.by(() => {
		const cats = new Set(containerLibrary.templates.map(t => t.category));
		return ['all', ...Array.from(cats)];
	});

	const filtered = $derived.by(() => {
		if (filterCategory === 'all') return containerLibrary.templates;
		return containerLibrary.templates.filter(t => t.category === filterCategory);
	});
</script>

<div class="space-y-4">
	<div class="flex gap-2">
		{#each categories as cat}
			<button onclick={() => filterCategory = cat}
				class="px-3 py-1 text-sm rounded-full transition-colors
					{filterCategory === cat ? 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-400' : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'}">
				{cat.charAt(0).toUpperCase() + cat.slice(1)}
			</button>
		{/each}
	</div>
	<div class="space-y-3">
		{#each filtered as template (template.id)}
			<TemplateCard {template} {onUse} />
		{/each}
	</div>
</div>
