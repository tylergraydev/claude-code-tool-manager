<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { mcpLibrary, projectsStore, subagentLibrary, skillLibrary, hookLibrary, commandLibrary, profileLibrary, statuslineLibrary } from '$lib/stores';
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
			label: 'Projects',
			value: projectCount,
			icon: FolderOpen,
			color: 'bg-blue-500',
			href: '/projects'
		},
		{
			label: 'MCPs',
			value: mcpCount,
			subtitle: `${globalMcpCount} global`,
			icon: Plug,
			color: 'bg-purple-500',
			href: '/library'
		},
		{
			label: 'Subagents',
			value: subagentCount,
			icon: Bot,
			color: 'bg-green-500',
			href: '/subagents'
		},
		{
			label: 'Commands',
			value: commandCount,
			subtitle: `${globalCommandCount} global`,
			icon: Terminal,
			color: 'bg-amber-500',
			href: '/commands'
		},
		{
			label: 'Skills',
			value: skillCount,
			icon: Sparkles,
			color: 'bg-purple-400',
			href: '/skills'
		},
		{
			label: 'Hooks',
			value: hookCount,
			subtitle: `${globalHookCount} global`,
			icon: Zap,
			color: 'bg-rose-500',
			href: '/hooks'
		},
		{
			label: 'Profiles',
			value: profileCount,
			subtitle: activeProfileName ? `Active: ${activeProfileName}` : 'None active',
			icon: Layers,
			color: 'bg-indigo-500',
			href: '/profiles'
		},
		{
			label: 'Status Line',
			value: statuslineCount,
			subtitle: activeStatusLineName ? `Active: ${activeStatusLineName}` : 'None active',
			icon: PanelBottom,
			color: 'bg-teal-500',
			href: '/statusline'
		}
	]);

	// Quick links with icons
	const quickLinks = [
		{ label: 'Add MCP', href: '/library', description: 'Add a new MCP server to your library', icon: Library },
		{ label: 'Manage Projects', href: '/projects', description: 'Configure MCPs for your projects', icon: FolderOpen },
		{ label: 'Browse Marketplace', href: '/marketplace', description: 'Discover community MCPs and tools', icon: Store },
		{ label: 'Settings', href: '/settings', description: 'Configure application settings', icon: Settings }
	];

	// Global config mini-cards
	const globalConfig = $derived([
		{
			label: 'Global MCPs',
			value: globalMcpCount,
			subtitle: 'MCP servers',
			icon: Plug,
			color: 'text-purple-500'
		},
		{
			label: 'Global Subagents',
			value: subagentLibrary.globalSubAgents.length,
			subtitle: 'Custom agents',
			icon: Bot,
			color: 'text-green-500'
		},
		{
			label: 'Global Commands',
			value: globalCommandCount,
			subtitle: 'Slash commands',
			icon: Terminal,
			color: 'text-amber-500'
		},
		{
			label: 'Global Hooks',
			value: globalHookCount,
			subtitle: 'Event hooks',
			icon: Zap,
			color: 'text-rose-500'
		}
	]);
</script>

<Header title="Dashboard" subtitle="Overview of your Claude Code configuration" />

<div class="flex-1 overflow-auto p-6 space-y-6">
	<!-- Stats Grid -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">Overview</h2>
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
						<span>View all</span>
						<ArrowRight class="w-3 h-3 ml-1 group-hover:translate-x-0.5 transition-transform" />
					</div>
				</button>
			{/each}
		</div>
	</section>

	<!-- Quick Links -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">Quick Actions</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
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

	<!-- Global Configuration -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">Global Configuration</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
			{#each globalConfig as item}
				<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
					<div class="flex items-center gap-3">
						<item.icon class="w-4 h-4 {item.color}" />
						<div>
							<p class="text-sm font-medium text-gray-900 dark:text-white">{item.value} {item.label}</p>
							<p class="text-xs text-gray-500 dark:text-gray-400">{item.subtitle}</p>
						</div>
					</div>
				</div>
			{/each}
		</div>
	</section>
</div>
