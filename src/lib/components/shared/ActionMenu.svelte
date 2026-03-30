<script lang="ts">
	import { MoreVertical } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		label?: string;
		children?: Snippet;
	};

	let { label = 'Actions', children }: Props = $props();
	let open = $state(false);

	export function close() {
		open = false;
	}

	function handleClickOutside(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.action-menu-container')) {
			open = false;
		}
	}
</script>

<svelte:window onclick={handleClickOutside} />

<div class="relative action-menu-container">
	<button
		class="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-500 dark:text-gray-400"
		onclick={(e) => { e.stopPropagation(); open = !open; }}
		aria-label={label}
		aria-haspopup="menu"
		aria-expanded={open}
	>
		<MoreVertical class="w-4 h-4" />
	</button>
	{#if open}
		<div
			class="absolute right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-50 min-w-[160px] py-1"
			role="menu"
		>
			{#if children}
				{@render children()}
			{/if}
		</div>
	{/if}
</div>
