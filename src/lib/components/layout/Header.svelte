<script lang="ts">
	import { RefreshCw, Moon, Sun } from 'lucide-svelte';
	import { mcpLibrary, projectsStore } from '$lib/stores';
	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		subtitle?: string;
		children?: Snippet;
	};

	let { title, subtitle, children }: Props = $props();
	let isDark = $state(true);

	async function handleRefresh() {
		await Promise.all([
			mcpLibrary.load(),
			projectsStore.loadProjects(),
			projectsStore.loadGlobalMcps()
		]);
	}

	function toggleTheme() {
		isDark = !isDark;
		document.documentElement.classList.toggle('dark', isDark);
	}
</script>

<header class="h-16 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-6 flex items-center justify-between">
	<div>
		<h2 class="text-xl font-semibold text-gray-900 dark:text-white">{title}</h2>
		{#if subtitle}
			<p class="text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
		{/if}
	</div>

	<div class="flex items-center gap-2">
		{#if children}
			{@render children()}
		{/if}

		<button
			onclick={handleRefresh}
			class="btn btn-ghost"
			title="Refresh data"
		>
			<RefreshCw class="w-4 h-4" />
		</button>

		<button
			onclick={toggleTheme}
			class="btn btn-ghost"
			title="Toggle theme"
		>
			{#if isDark}
				<Sun class="w-4 h-4" />
			{:else}
				<Moon class="w-4 h-4" />
			{/if}
		</button>
	</div>
</header>
