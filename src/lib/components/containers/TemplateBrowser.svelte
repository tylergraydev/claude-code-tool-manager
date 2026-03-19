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
				class="px-3 py-1 text-sm rounded-full"
				class:bg-primary-600={selectedCategory === 'all'}
				class:text-white={selectedCategory === 'all'}
				onclick={() => selectedCategory = 'all'}
			>All</button>
			{#each categories as cat}
				<button
					class="px-3 py-1 text-sm rounded-full"
					class:bg-primary-600={selectedCategory === cat}
					class:text-white={selectedCategory === cat}
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
