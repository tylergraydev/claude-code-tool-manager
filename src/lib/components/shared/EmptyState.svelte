<script lang="ts">
	import type { Snippet, Component } from 'svelte';

	type Props = {
		icon?: Component<any>;
		title: string;
		description?: string;
		children?: Snippet;
	};

	let { icon: Icon, title, description, children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 text-center">
	{#if Icon}
		<div class="w-12 h-12 rounded-xl bg-gray-100 dark:bg-gray-800 flex items-center justify-center mb-4">
			<Icon class="w-6 h-6 text-gray-400" />
		</div>
	{/if}
	<h3 class="text-lg font-medium text-gray-900 dark:text-white">{title}</h3>
	{#if description}
		<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{description}</p>
	{/if}
	{#if children}
		<div class="mt-4">
			{@render children()}
		</div>
	{/if}
</div>
