<script lang="ts">
	import { page } from '$app/stores';
	import { Library, FolderOpen, Settings, Plug, FileCode, Bot } from 'lucide-svelte';

	const navItems = [
		{ href: '/', label: 'Dashboard', icon: Plug },
		{ href: '/library', label: 'MCP Library', icon: Library },
		{ href: '/skills', label: 'Skills Library', icon: FileCode },
		{ href: '/subagents', label: 'Sub-Agents Library', icon: Bot },
		{ href: '/projects', label: 'Projects', icon: FolderOpen },
		{ href: '/settings', label: 'Global Settings', icon: Settings }
	];
</script>

<aside class="w-64 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
	<div class="p-4 border-b border-gray-200 dark:border-gray-700">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
				<Plug class="w-5 h-5 text-white" />
			</div>
			<div>
				<h1 class="font-semibold text-gray-900 dark:text-white">Claude Code</h1>
				<p class="text-xs text-gray-500 dark:text-gray-400">Tool Manager</p>
			</div>
		</div>
	</div>

	<nav class="flex-1 p-3 space-y-1">
		{#each navItems as item}
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
	</nav>

	<div class="p-4 border-t border-gray-200 dark:border-gray-700">
		<p class="text-xs text-gray-400 dark:text-gray-500">
			v1.0.0
		</p>
	</div>
</aside>
