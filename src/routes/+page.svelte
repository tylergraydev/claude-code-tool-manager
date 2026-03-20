<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { mcpLibrary, projectsStore, subagentLibrary, skillLibrary, hookLibrary, commandLibrary, profileLibrary, statuslineLibrary } from '$lib/stores';
	import { onboarding } from '$lib/stores/onboarding.svelte';
	import WelcomeHero from '$lib/components/onboarding/WelcomeHero.svelte';
	import { i18n } from '$lib/i18n';
	import { FolderOpen, Plug, Bot, Sparkles, Zap, ArrowRight, Terminal, Layers, PanelBottom, Library, Settings, Store } from 'lucide-svelte';
	import { goto } from '$app/navigation';

	// Derived counts
	const projectCount = $derived(projectsStore.projects.length);
	const mcpCount = $derived(mcpLibrary.mcps.length);
	const globalMcpCount = $derived(projectsStore.globalMcps.length);
	const subagentCount = $derived(subagentLibrary.subagents.length);
	const skillCount = $derived(skillLibrary.skills.length);
	const commandCount = $derived(commandLibrary.commands.length);
	const globalCommandCount = $derived(commandLibrary.globalCommands.length);
	const hookCount = $derived(hookLibrary.hooks.length);
	const globalHookCount = $derived(hookLibrary.globalHooks.length);
	const profileCount = $derived(profileLibrary.profiles.length);
	const activeProfileName = $derived(profileLibrary.activeProfile?.name);
	const statuslineCount = $derived(statuslineLibrary.statuslines.length);
	const activeStatusLineName = $derived(statuslineLibrary.activeStatusLine?.name);

	const stats = $derived([
		{
			label: i18n.t('dashboard.projects'),
			value: projectCount,
			icon: FolderOpen,
			color: 'bg-blue-500',
			href: '/projects'
		},
		{
			label: i18n.t('dashboard.mcps'),
			value: mcpCount,
			subtitle: `${globalMcpCount} ${i18n.t('dashboard.global')}`,
			icon: Plug,
			color: 'bg-purple-500',
			href: '/library'
		},
		{
			label: i18n.t('dashboard.subagents'),
			value: subagentCount,
			icon: Bot,
			color: 'bg-green-500',
			href: '/subagents'
		},
		{
			label: i18n.t('dashboard.commands'),
			value: commandCount,
			subtitle: `${globalCommandCount} ${i18n.t('dashboard.global')}`,
			icon: Terminal,
			color: 'bg-amber-500',
			href: '/commands'
		},
		{
			label: i18n.t('dashboard.skills'),
			value: skillCount,
			icon: Sparkles,
			color: 'bg-purple-400',
			href: '/skills'
		},
		{
			label: i18n.t('dashboard.hooks'),
			value: hookCount,
			subtitle: `${globalHookCount} ${i18n.t('dashboard.global')}`,
			icon: Zap,
			color: 'bg-rose-500',
			href: '/hooks'
		},
		{
			label: i18n.t('dashboard.profiles'),
			value: profileCount,
			subtitle: activeProfileName ? i18n.t('dashboard.activeProfile', { name: activeProfileName }) : i18n.t('dashboard.noneActive'),
			icon: Layers,
			color: 'bg-indigo-500',
			href: '/profiles'
		},
		{
			label: i18n.t('dashboard.statusLine'),
			value: statuslineCount,
			subtitle: activeStatusLineName ? i18n.t('dashboard.activeProfile', { name: activeStatusLineName }) : i18n.t('dashboard.noneActive'),
			icon: PanelBottom,
			color: 'bg-teal-500',
			href: '/statusline'
		}
	]);

	// Quick links with icons
	const quickLinks = [
		{ label: i18n.t('dashboard.addMcp'), href: '/library', description: i18n.t('dashboard.addMcpDesc'), icon: Library },
		{ label: i18n.t('dashboard.manageProjects'), href: '/projects', description: i18n.t('dashboard.manageProjectsDesc'), icon: FolderOpen },
		{ label: i18n.t('dashboard.browseMarketplace'), href: '/marketplace', description: i18n.t('dashboard.browseMarketplaceDesc'), icon: Store },
		{ label: i18n.t('dashboard.settings'), href: '/settings', description: i18n.t('dashboard.settingsDesc'), icon: Settings }
	];

	// Global config mini-cards
	const globalConfig = $derived([
		{
			label: i18n.t('dashboard.globalMcps'),
			value: globalMcpCount,
			subtitle: i18n.t('dashboard.mcpServers'),
			icon: Plug,
			color: 'text-purple-500'
		},
		{
			label: i18n.t('dashboard.globalSubagents'),
			value: subagentLibrary.globalSubAgents.length,
			subtitle: i18n.t('dashboard.customAgents'),
			icon: Bot,
			color: 'text-green-500'
		},
		{
			label: i18n.t('dashboard.globalCommands'),
			value: globalCommandCount,
			subtitle: i18n.t('dashboard.slashCommands'),
			icon: Terminal,
			color: 'text-amber-500'
		},
		{
			label: i18n.t('dashboard.globalHooks'),
			value: globalHookCount,
			subtitle: i18n.t('dashboard.eventHooks'),
			icon: Zap,
			color: 'text-rose-500'
		}
	]);
</script>

<Header title={i18n.t('page.dashboard.title')} subtitle={i18n.t('page.dashboard.subtitle')} />

<div class="flex-1 overflow-auto p-6">
	<div class="max-w-6xl space-y-8">
		<!-- Onboarding -->
		{#if onboarding.showOnboarding}
			<WelcomeHero {projectCount} {mcpCount} {globalMcpCount} />
		{/if}

		<!-- Stats Grid -->
		<section>
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">{i18n.t('dashboard.overview')}</h2>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
				{#each stats as stat}
					<button
						onclick={() => goto(stat.href)}
						class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-md transition-all text-left group"
					>
						<div class="flex items-start justify-between">
							<div>
								<p class="text-sm font-medium text-gray-500 dark:text-gray-400">{stat.label}</p>
								<p class="text-2xl font-bold text-gray-900 dark:text-white mt-0.5">{stat.value}</p>
								{#if stat.subtitle}
									<p class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{stat.subtitle}</p>
								{/if}
							</div>
							<div class="{stat.color} p-2 rounded-lg">
								<stat.icon class="w-4 h-4 text-white" />
							</div>
						</div>
						<div class="mt-2 flex items-center text-xs text-gray-500 dark:text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-300 transition-colors">
							<span>{i18n.t('common.viewAll')}</span>
							<ArrowRight class="w-3 h-3 ml-1 group-hover:translate-x-0.5 transition-transform" />
						</div>
					</button>
				{/each}
			</div>
		</section>

		<!-- Quick Actions + Global Config side by side -->
		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
			<!-- Quick Actions — takes 2 columns -->
			<section class="lg:col-span-2">
				<h2 class="text-sm font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3">{i18n.t('dashboard.quickActions')}</h2>
				<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
					{#each quickLinks as link}
						<a
							href={link.href}
							class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-md transition-all group"
						>
							<div class="flex items-center gap-3 mb-2">
								<div class="p-1.5 rounded-lg bg-gray-100 dark:bg-gray-700 group-hover:bg-blue-50 dark:group-hover:bg-blue-900/30 transition-colors">
									<link.icon class="w-4 h-4 text-gray-500 dark:text-gray-400 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors" />
								</div>
								<p class="font-medium text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
									{link.label}
								</p>
							</div>
							<p class="text-sm text-gray-500 dark:text-gray-400">{link.description}</p>
						</a>
					{/each}
				</div>
			</section>

			<!-- Global Configuration — takes 1 column -->
			<section>
				<h2 class="text-sm font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3">{i18n.t('dashboard.globalConfig')}</h2>
				<div class="space-y-2">
					{#each globalConfig as item}
						<div class="flex items-center gap-3 px-4 py-3 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
							<item.icon class="w-4 h-4 {item.color} flex-shrink-0" />
							<div class="min-w-0">
								<p class="text-sm font-medium text-gray-900 dark:text-white">{item.value} {item.label}</p>
								<p class="text-xs text-gray-500 dark:text-gray-400">{item.subtitle}</p>
							</div>
						</div>
					{/each}
				</div>
			</section>
		</div>
	</div>
</div>
