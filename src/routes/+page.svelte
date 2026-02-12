<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { mcpLibrary, projectsStore, subagentLibrary, skillLibrary, hookLibrary, commandLibrary, profileLibrary, statuslineLibrary } from '$lib/stores';
	import { FolderOpen, Plug, Bot, Sparkles, Zap, Globe, ArrowRight, Terminal, Layers, PanelBottom } from 'lucide-svelte';
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

	// Quick links
	const quickLinks = [
		{ label: 'Add MCP', href: '/library', description: 'Add a new MCP server to your library' },
		{ label: 'Manage Projects', href: '/projects', description: 'Configure MCPs for your projects' },
		{ label: 'Browse Marketplace', href: '/marketplace', description: 'Discover community MCPs and tools' },
		{ label: 'Settings', href: '/settings', description: 'Configure application settings' }
	];
</script>

<Header title="Dashboard" subtitle="Overview of your Claude Code configuration" />

<div class="flex-1 overflow-auto p-6 space-y-8">
	<!-- Stats Grid -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Overview</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
			{#each stats as stat}
				<button
					onclick={() => goto(stat.href)}
					class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-md transition-all text-left group"
				>
					<div class="flex items-start justify-between">
						<div>
							<p class="text-sm font-medium text-gray-500 dark:text-gray-400">{stat.label}</p>
							<p class="text-3xl font-bold text-gray-900 dark:text-white mt-1">{stat.value}</p>
							{#if stat.subtitle}
								<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">{stat.subtitle}</p>
							{/if}
						</div>
						<div class="{stat.color} p-2.5 rounded-lg">
							<stat.icon class="w-5 h-5 text-white" />
						</div>
					</div>
					<div class="mt-3 flex items-center text-sm text-gray-500 dark:text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-300 transition-colors">
						<span>View all</span>
						<ArrowRight class="w-4 h-4 ml-1 group-hover:translate-x-0.5 transition-transform" />
					</div>
				</button>
			{/each}
		</div>
	</section>

	<!-- Quick Links -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Actions</h2>
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
			{#each quickLinks as link}
				<a
					href={link.href}
					class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 hover:border-blue-300 dark:hover:border-blue-600 hover:shadow-md transition-all group"
				>
					<p class="font-medium text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
						{link.label}
					</p>
					<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{link.description}</p>
				</a>
			{/each}
		</div>
	</section>

	<!-- Global Configuration Summary -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Global Configuration</h2>
		<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5">
			<div class="flex items-center gap-2 mb-4">
				<Globe class="w-5 h-5 text-gray-400" />
				<span class="text-sm text-gray-500 dark:text-gray-400">
					Items enabled globally apply to all Claude Code sessions
				</span>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
				<div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
					<Plug class="w-4 h-4 text-purple-500" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">{globalMcpCount} Global MCPs</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">MCP servers</p>
					</div>
				</div>
				<div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
					<Bot class="w-4 h-4 text-green-500" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">{subagentLibrary.globalSubAgents.length} Global Subagents</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Custom agents</p>
					</div>
				</div>
				<div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
					<Terminal class="w-4 h-4 text-amber-500" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">{globalCommandCount} Global Commands</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Slash commands</p>
					</div>
				</div>
				<div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
					<Zap class="w-4 h-4 text-rose-500" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">{globalHookCount} Global Hooks</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Event hooks</p>
					</div>
				</div>
			</div>
		</div>
	</section>
</div>
