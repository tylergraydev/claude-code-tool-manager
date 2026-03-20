<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		variant?: 'default' | 'success' | 'warning' | 'error';
		title?: string;
		children?: Snippet;
	};

	let { variant = 'default', title, children }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: 'bg-gray-100 text-gray-700',
		success: 'bg-green-100 text-green-700',
		warning: 'bg-yellow-100 text-yellow-700',
		error: 'bg-red-100 text-red-700'
	};
</script>

<span
	class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {variantClasses[variant] ?? ''}"
	{title}
	class:cursor-help={!!title}
>
	{#if children}
		{@render children()}
	{/if}
</span>
