<script lang="ts">
	import { onMount } from 'svelte';
	import { RefreshCw, Moon, Sun, Languages } from 'lucide-svelte';
	import { mcpLibrary, projectsStore, notifications, i18n } from '$lib/stores';
	import type { Snippet } from 'svelte';

	type Props = {
		title: string;
		subtitle?: string;
		children?: Snippet;
	};

	let { title, subtitle, children }: Props = $props();
	let isDark = $state(true);
	let isRefreshing = $state(false);

	onMount(() => {
		isDark = document.documentElement.classList.contains('dark');
	});

	async function handleRefresh() {
		if (isRefreshing) return;
		isRefreshing = true;
		try {
			await Promise.all([
				mcpLibrary.load(),
				projectsStore.loadProjects(),
				projectsStore.loadGlobalMcps()
			]);
		} catch (e) {
			notifications.error('Failed to refresh data');
			console.error('[Header] Refresh failed:', e);
		} finally {
			isRefreshing = false;
		}
	}

	function toggleTheme() {
		isDark = !isDark;
		document.documentElement.classList.toggle('dark', isDark);
		try {
			localStorage.setItem('theme', isDark ? 'dark' : 'light');
		} catch {}
	}
</script>

<header class="h-16 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-6 flex items-center justify-between">
	<div>
		<h1 class="text-xl font-semibold text-gray-900 dark:text-white">{title}</h1>
		{#if subtitle}
			<p class="text-sm text-gray-500 dark:text-gray-400">{subtitle}</p>
		{/if}
	</div>

	<div class="flex items-center gap-2">
		{#if children}
			{@render children()}
		{/if}

		<button
			onclick={() => i18n.setLocale(i18n.nextLocale)}
			class="btn btn-ghost text-xs font-medium"
			title={i18n.t('header.switchLanguage')}
		>
			<Languages class="w-4 h-4" />
			{i18n.currentLabel}
		</button>

		<button
			onclick={handleRefresh}
			class="btn btn-ghost"
			aria-label={isRefreshing ? 'Refreshing data' : 'Refresh data'}
			title={i18n.t('header.refresh')}
			disabled={isRefreshing}
		>
			<RefreshCw class="w-4 h-4 {isRefreshing ? 'animate-spin' : ''}" aria-hidden="true" />
		</button>

		<button
			onclick={toggleTheme}
			class="btn btn-ghost"
			aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
			title={i18n.t('header.toggleTheme')}
		>
			{#if isDark}
				<Sun class="w-4 h-4" aria-hidden="true" />
			{:else}
				<Moon class="w-4 h-4" aria-hidden="true" />
			{/if}
		</button>
	</div>
</header>
