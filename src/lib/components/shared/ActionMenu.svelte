<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		label?: string;
		children?: Snippet;
	};

	let { label = 'Actions', children }: Props = $props();

	let open = $state(false);
</script>

<div class="relative">
	<button
		class="p-1 hover:bg-gray-100 rounded"
		onclick={() => (open = !open)}
		aria-label={label}
		aria-haspopup="menu"
		aria-expanded={open}
	>
		<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
			<circle cx="12" cy="5" r="2" />
			<circle cx="12" cy="12" r="2" />
			<circle cx="12" cy="19" r="2" />
		</svg>
	</button>
	{#if open}
		<div class="absolute right-0 mt-1 bg-white border rounded-md shadow-lg z-50 min-w-[160px]" role="menu">
			{#if children}
				{@render children()}
			{/if}
		</div>
	{/if}
</div>
