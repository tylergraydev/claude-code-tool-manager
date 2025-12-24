<script lang="ts">
	import type { Mcp } from '$lib/types';
	import { Plug, Globe, Server, MoreVertical, Edit, Copy, Trash2, Play, Lock, Radio } from 'lucide-svelte';

	type Props = {
		mcp: Mcp;
		showActions?: boolean;
		showGatewayToggle?: boolean;
		isInGateway?: boolean;
		onEdit?: (mcp: Mcp) => void;
		onDelete?: (mcp: Mcp) => void;
		onDuplicate?: (mcp: Mcp) => void;
		onTest?: (mcp: Mcp) => void;
		onGatewayToggle?: (mcp: Mcp, enabled: boolean) => void;
	};

	let {
		mcp,
		showActions = true,
		showGatewayToggle = false,
		isInGateway = false,
		onEdit,
		onDelete,
		onDuplicate,
		onTest,
		onGatewayToggle
	}: Props = $props();

	let showMenu = $state(false);

	// System MCPs are readonly
	const isSystemMcp = mcp.source === 'system';

	const typeIcons = {
		stdio: Plug,
		sse: Globe,
		http: Server
	};

	const typeColors = {
		stdio: 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400',
		sse: 'bg-green-100 text-green-600 dark:bg-green-900/50 dark:text-green-400',
		http: 'bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400'
	};

	function closeMenu() {
		showMenu = false;
	}
</script>

<svelte:window onclick={closeMenu} />

<div class="card group relative hover:shadow-md transition-all duration-200">
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl {typeColors[mcp.type]} flex items-center justify-center">
			<svelte:component this={typeIcons[mcp.type]} class="w-5 h-5" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{mcp.name}
				</h3>
				{#if isSystemMcp}
					<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
						<Lock class="w-2.5 h-2.5" />
						System
					</span>
				{:else if mcp.source === 'auto-detected'}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300">
						Auto
					</span>
				{/if}
			</div>

			{#if mcp.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{mcp.description}
				</p>
			{/if}

			<div class="flex items-center gap-2 mt-2 flex-wrap">
				<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
					{mcp.type}
				</span>

				{#if mcp.type === 'stdio' && mcp.command}
					<span class="text-xs text-gray-400 dark:text-gray-500 font-mono truncate max-w-[150px]">
						{mcp.command}
					</span>
				{:else if mcp.url}
					<span class="text-xs text-gray-400 dark:text-gray-500 truncate max-w-[150px]">
						{new URL(mcp.url).hostname}
					</span>
				{/if}

				{#if showGatewayToggle && isInGateway}
					<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300">
						<Radio class="w-3 h-3" />
						Gateway
					</span>
				{/if}
			</div>

			{#if showGatewayToggle}
				<div class="flex items-center gap-2 mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
					<button
						onclick={(e) => {
							e.stopPropagation();
							onGatewayToggle?.(mcp, !isInGateway);
						}}
						class="flex items-center gap-2 text-xs font-medium transition-colors
							{isInGateway
								? 'text-amber-600 dark:text-amber-400 hover:text-amber-700 dark:hover:text-amber-300'
								: 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'}"
						title={isInGateway ? 'Remove from Gateway' : 'Add to Gateway'}
					>
						<Radio class="w-4 h-4" />
						{isInGateway ? 'In Gateway' : 'Add to Gateway'}
					</button>
				</div>
			{/if}
		</div>

		{#if showActions}
			<div class="relative">
				<button
					onclick={(e) => {
						e.stopPropagation();
						showMenu = !showMenu;
					}}
					class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
				>
					<MoreVertical class="w-4 h-4" />
				</button>

				{#if showMenu}
					<div
						class="absolute right-0 top-full mt-1 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-10"
						onclick={(e) => e.stopPropagation()}
					>
						{#if onTest}
							<button
								onclick={() => {
									onTest(mcp);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Play class="w-4 h-4" />
								Test
							</button>
						{/if}
						{#if onEdit && !isSystemMcp}
							<button
								onclick={() => {
									onEdit(mcp);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Edit class="w-4 h-4" />
								Edit
							</button>
						{/if}
						{#if onDuplicate && !isSystemMcp}
							<button
								onclick={() => {
									onDuplicate(mcp);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Copy class="w-4 h-4" />
								Duplicate
							</button>
						{/if}
						{#if onDelete && !isSystemMcp}
							<button
								onclick={() => {
									onDelete(mcp);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
							>
								<Trash2 class="w-4 h-4" />
								Delete
							</button>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>
