<script lang="ts">
	import type { Snippet, Component } from 'svelte';

	type Props = {
		variant?: 'default' | 'success' | 'warning' | 'error' | 'system' | 'auto';
		title?: string;
		icon?: Component<any>;
		children?: Snippet;
	};

	let { variant = 'default', title, icon: Icon, children }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300',
		success: 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300',
		warning: 'bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300',
		error: 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300',
		system: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300',
		auto: 'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/50 dark:text-cyan-300'
	};
</script>

<span
	class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium {variantClasses[variant] ?? variantClasses.default}"
	{title}
	class:cursor-help={!!title}
>
	{#if Icon}
		<Icon class="w-3 h-3" />
	{/if}
	{#if children}
		{@render children()}
	{/if}
</span>
