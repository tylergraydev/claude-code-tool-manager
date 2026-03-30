<script lang="ts">
	import { page } from '$app/stores';
	import { Library, FolderOpen, Settings, Bot, Store, Zap, Terminal, Sparkles, Layers, PanelBottom, Shield, FileText, Plug, BarChart3, TrendingUp, FolderSearch, GitCompareArrows, Container, PanelLeftClose, PanelLeftOpen, BookOpen } from 'lucide-svelte';
	import { onMount, onDestroy } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import TodayUsageWidget from './TodayUsageWidget.svelte';
	import { sessionStore, i18n } from '$lib/stores';

	let appVersion = $state('');
	let collapsed = $state(false);
	let manualOverride = $state(false);
	let windowWidth = $state(window.innerWidth);

	const AUTO_COLLAPSE_WIDTH = 860;

	let resizeRaf: number | null = null;

	function handleResize() {
		if (resizeRaf) return;
		resizeRaf = requestAnimationFrame(() => {
			resizeRaf = null;
			windowWidth = window.innerWidth;
			if (!manualOverride) {
				collapsed = windowWidth < AUTO_COLLAPSE_WIDTH;
			}
		});
	}

	function toggleCollapse() {
		manualOverride = true;
		collapsed = !collapsed;
	}

	onMount(async () => {
		window.addEventListener('resize', handleResize);
		handleResize();

		try {
			appVersion = await getVersion();
		} catch {
			appVersion = '1.0.0';
		}
		if (sessionStore.projects.length === 0) {
			sessionStore.loadProjects();
		}
	});

	onDestroy(() => {
		window.removeEventListener('resize', handleResize);
		if (resizeRaf) cancelAnimationFrame(resizeRaf);
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
				{ href: '/hooks', label: i18n.t('nav.hooks'), icon: Zap },
				{ href: '/rules', label: 'Rules', icon: BookOpen },
				{ href: '/containers', label: 'Containers', icon: Container }
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

<aside
	class="border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col transition-[width] duration-200 ease-in-out {collapsed ? 'w-16' : 'w-56'}"
>
	<!-- Header -->
	<div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center {collapsed ? 'justify-center' : 'gap-3'}">
		<div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center shrink-0">
			<Plug class="w-5 h-5 text-white" />
		</div>
		{#if !collapsed}
			<div class="overflow-hidden">
				<h1 class="font-semibold text-gray-900 dark:text-white whitespace-nowrap">{i18n.t('app.name')}</h1>
				<p class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">{i18n.t('app.tagline')}</p>
			</div>
		{/if}
	</div>

	<!-- Navigation -->
	<nav aria-label="Main navigation" class="flex-1 overflow-y-auto p-3 {collapsed ? 'px-2' : ''}">
		{#each navGroups as group, groupIndex}
			{#if groupIndex > 0}
				<div class="mt-3"></div>
			{/if}
			{#if !collapsed}
				<p class="text-[10px] font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-400 px-3 mb-1.5">
					{group.label}
				</p>
			{:else}
				<div class="h-px bg-gray-200 dark:bg-gray-700 mx-2 mb-2"></div>
			{/if}
			<div class="space-y-0.5">
				{#each group.items as item}
					{@const isActive = $page.url.pathname === item.href ||
						(item.href !== '/' && $page.url.pathname.startsWith(item.href))}
					<a
						href={item.href}
						title={collapsed ? item.label : undefined}
						class="flex items-center rounded-lg text-sm font-medium transition-colors
							{collapsed ? 'justify-center px-0 py-2' : 'gap-3 px-3 py-2'}
							{isActive
								? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
					>
						<item.icon class="w-5 h-5 shrink-0" />
						{#if !collapsed}
							<span class="whitespace-nowrap overflow-hidden">{item.label}</span>
						{/if}
					</a>
				{/each}
			</div>
		{/each}
	</nav>

	<!-- Footer -->
	<div class="border-t border-gray-200 dark:border-gray-700 p-3 {collapsed ? 'px-2' : ''}">
		{#if !collapsed}
			<TodayUsageWidget />
		{/if}
		<a
			href="/settings"
			title={collapsed ? i18n.t('nav.settings') : undefined}
			class="flex items-center rounded-lg text-sm font-medium transition-colors
				{collapsed ? 'justify-center px-0 py-2' : 'gap-3 px-3 py-2'}
				{isSettingsActive
					? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
					: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
		>
			<Settings class="w-5 h-5 shrink-0" />
			{#if !collapsed}
				<span>{i18n.t('nav.settings')}</span>
			{/if}
		</a>

		<!-- Collapse toggle -->
		<button
			onclick={toggleCollapse}
			class="flex items-center w-full rounded-lg text-sm font-medium transition-colors mt-1
				{collapsed ? 'justify-center px-0 py-2' : 'gap-3 px-3 py-2'}
				text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700/50 dark:hover:text-gray-300"
			title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			{#if collapsed}
				<PanelLeftOpen class="w-5 h-5 shrink-0" />
			{:else}
				<PanelLeftClose class="w-5 h-5 shrink-0" />
				<span>Collapse</span>
			{/if}
		</button>

		{#if !collapsed}
			<p class="text-xs text-gray-400 dark:text-gray-500 px-3 mt-2">
				{appVersion ? `v${appVersion}` : ''}
			</p>
		{/if}
	</div>
</aside>
