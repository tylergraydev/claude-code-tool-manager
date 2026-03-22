<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import TemplateCard from './TemplateCard.svelte';

	type Props = {
		onUse: (template: any) => void;
	};

	let { onUse }: Props = $props();

	let selectedCategory = $state('all');

	const categories = $derived.by(() => {
		const cats = new Set<string>();
		for (const t of containerLibrary.templates) {
			if (t.category) cats.add(t.category);
		}
		return Array.from(cats);
	});

	const filteredTemplates = $derived(
		selectedCategory === 'all'
			? containerLibrary.templates
			: containerLibrary.templates.filter((t: any) => t.category === selectedCategory)
	);

	function capitalize(s: string): string {
		return s.charAt(0).toUpperCase() + s.slice(1);
	}
</script>

<div class="space-y-4">
	{#if containerLibrary.templates.length > 0}
		<div class="flex gap-2 flex-wrap">
			<button
				class="px-3 py-1.5 text-sm rounded-md border transition-colors {selectedCategory === 'all'
					? 'bg-primary-50 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-700 dark:text-primary-300'
					: 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-500'}"
				onclick={() => selectedCategory = 'all'}
			>All</button>
			{#each categories as cat}
				<button
					class="px-3 py-1.5 text-sm rounded-md border transition-colors {selectedCategory === cat
						? 'bg-primary-50 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-700 dark:text-primary-300'
						: 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-500'}"
					onclick={() => selectedCategory = cat}
				>{capitalize(cat)}</button>
			{/each}
		</div>

		<div class="space-y-3">
			{#each filteredTemplates as template (template.id)}
				<TemplateCard {template} {onUse} />
			{/each}
		</div>
	{/if}
</div>
