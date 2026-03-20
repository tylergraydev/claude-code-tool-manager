<script lang="ts">
	import type { Component } from 'svelte';

	type Props = {
		icon?: Component<any>;
		label: string;
		variant?: 'default' | 'danger';
		onclick?: () => void;
	};

	let { icon, label, variant = 'default', onclick }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: 'text-gray-700 hover:bg-gray-100',
		danger: 'text-red-600 hover:bg-red-50'
	};
</script>

<button
	class="w-full text-left px-3 py-2 text-sm flex items-center gap-2 transition-colors {variantClasses[variant]}"
	{onclick}
	role="menuitem"
>
	{#if icon}
		{@const Icon = icon}
		<Icon class="w-4 h-4" />
	{/if}
	{label}
</button>
