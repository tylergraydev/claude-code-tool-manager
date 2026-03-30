<script lang="ts">
	import type { Component } from 'svelte';

	type Props = {
		icon?: Component<any>;
		label: string;
		variant?: 'default' | 'danger';
		onclick?: () => void;
	};

	let { icon: Icon, label, variant = 'default', onclick }: Props = $props();
</script>

<button
	class="w-full flex items-center gap-2 text-left px-3 py-2 text-sm transition-colors
		{variant === 'danger'
			? 'text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20'
			: 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
	{onclick}
	role="menuitem"
>
	{#if Icon}
		<Icon class="w-4 h-4" />
	{/if}
	{label}
</button>
