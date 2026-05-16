<script lang="ts">
	// `icon` is intentionally typed loosely. lucide-svelte 1.x still exports
	// icons as the legacy `SvelteComponentTyped` class shape, which doesn't
	// satisfy Svelte 5's `Component<Props>` type — but renders fine at runtime.
	// Accepting both shapes via the broad type avoids forcing a wrapper around
	// every icon library import.
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	type IconLike = any;

	type Props = {
		icon?: IconLike;
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
