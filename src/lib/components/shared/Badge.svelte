<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		variant?: 'default' | 'success' | 'warning' | 'error' | 'info' | 'auto';
		title?: string;
		children?: Snippet;
	};

	let { variant = 'default', title, children }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
		success: 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300',
		warning: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-300',
		error: 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300',
		info: 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300',
		auto: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300'
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
