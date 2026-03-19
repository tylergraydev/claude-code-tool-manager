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
		<div class="mb-4 text-gray-400">
			<Icon size={48} />
		</div>
	{/if}
	<h3 class="text-lg font-medium text-gray-900">{title}</h3>
	{#if description}
		<p class="mt-2 text-sm text-gray-500">{description}</p>
	{/if}
	{#if children}
		<div class="mt-4">
			{@render children()}
		</div>
	{/if}
</div>
