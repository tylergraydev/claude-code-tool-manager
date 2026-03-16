<script lang="ts">
	import { page } from '$app/stores';
	import { Library, FolderOpen, Settings, Bot, Store, Zap, Terminal, Sparkles, Layers, PanelBottom, Shield, FileText, Plug, BarChart3, TrendingUp, FolderSearch, GitCompareArrows } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import TodayUsageWidget from './TodayUsageWidget.svelte';
	import { sessionStore, i18n } from '$lib/stores';

	let appVersion = $state('');

	onMount(async () => {
		try {
			appVersion = await getVersion();
		} catch {
			appVersion = '1.0.0';
		}
		// Load session projects for the usage widget
		if (sessionStore.projects.length === 0) {
			sessionStore.loadProjects();
		}
	});

	interface NavItem {
		href: string;
		label: string;
		icon: typeof Plug;
	}

	interface NavGroup {
		label: string;
		items: NavItem[];
	}

	const isSettingsActive = $derived($page.url.pathname === '/settings' || $page.url.pathname.startsWith('/settings'));

	const navGroups: NavGroup[] = $derived([
		{
			label: i18n.t('nav.core'),
			items: [
				{ href: '/', label: i18n.t('nav.dashboard'), icon: Plug },
				{ href: '/projects', label: i18n.t('nav.projects'), icon: FolderOpen }
			]
		},
		{
			label: i18n.t('nav.tools'),
			items: [
				{ href: '/library', label: i18n.t('nav.mcps'), icon: Library },
				{ href: '/subagents', label: i18n.t('nav.agents'), icon: Bot },
				{ href: '/skills', label: i18n.t('nav.skills'), icon: Sparkles },
				{ href: '/commands', label: i18n.t('nav.commands'), icon: Terminal },
				{ href: '/hooks', label: i18n.t('nav.hooks'), icon: Zap }
			]
		},
		{
			label: i18n.t('nav.configure'),
			items: [
				{ href: '/profiles', label: i18n.t('nav.profiles'), icon: Layers },
				{ href: '/statusline', label: i18n.t('nav.statusLine'), icon: PanelBottom },
				{ href: '/permissions', label: i18n.t('nav.permissions'), icon: Shield },
				{ href: '/memory', label: i18n.t('nav.memory'), icon: FileText },
				{ href: '/marketplace', label: i18n.t('nav.marketplace'), icon: Store }
			]
		},
		{
			label: i18n.t('nav.insights'),
			items: [
				{ href: '/analytics', label: i18n.t('nav.analytics'), icon: BarChart3 },
				{ href: '/insights', label: i18n.t('nav.insights'), icon: TrendingUp },
				{ href: '/sessions', label: i18n.t('nav.sessions'), icon: FolderSearch },
				{ href: '/comparison', label: i18n.t('nav.comparison'), icon: GitCompareArrows }
			]
		}
	]);
</script>

<aside class="w-56 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
	<div class="p-4 border-b border-gray-200 dark:border-gray-700">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
				<Plug class="w-5 h-5 text-white" />
			</div>
			<div>
				<h1 class="font-semibold text-gray-900 dark:text-white">{i18n.t('app.name')}</h1>
				<p class="text-xs text-gray-500 dark:text-gray-400">{i18n.t('app.tagline')}</p>
			</div>
		</div>
	</div>

	<nav class="flex-1 overflow-y-auto p-3">
		{#each navGroups as group, groupIndex}
			{#if groupIndex > 0}
				<div class="mt-3"></div>
			{/if}
			<p class="text-[10px] font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 px-3 mb-1.5">
				{group.label}
			</p>
			<div class="space-y-0.5">
				{#each group.items as item}
					{@const isActive = $page.url.pathname === item.href ||
						(item.href !== '/' && $page.url.pathname.startsWith(item.href))}
					<a
						href={item.href}
						class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
							{isActive
								? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
					>
						<svelte:component this={item.icon} class="w-5 h-5" />
						{item.label}
					</a>
				{/each}
			</div>
		{/each}
	</nav>

	<div class="border-t border-gray-200 dark:border-gray-700 p-3">
		<TodayUsageWidget />
		<a
			href="/settings"
			class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
				{isSettingsActive
					? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
					: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
		>
			<Settings class="w-5 h-5" />
			{i18n.t('nav.settings')}
		</a>
		<p class="text-xs text-gray-400 dark:text-gray-500 px-3 mt-2">
			{appVersion ? `v${appVersion}` : ''}
		</p>
	</div>
</aside>
