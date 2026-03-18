<script lang="ts">
	import type { Mcp } from '$lib/types';
	import { Plug, Globe, Server, Edit, Copy, Trash2, Play, Lock, Radio } from 'lucide-svelte';
	import { ActionMenu, ActionMenuItem, FavoriteButton, Badge } from '$lib/components/shared';

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
		onFavoriteToggle?: (mcp: Mcp, favorite: boolean) => void;
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
		onGatewayToggle,
		onFavoriteToggle
	}: Props = $props();

	let actionMenu: ActionMenu;

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

	const TypeIcon = typeIcons[mcp.type];
</script>

<div class="card group relative hover:shadow-md transition-shadow duration-200">
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl {typeColors[mcp.type]} flex items-center justify-center">
			<TypeIcon class="w-5 h-5" aria-hidden="true" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{mcp.name}
				</h3>
				{#if isSystemMcp}
					<Badge variant="system" icon={Lock}>System</Badge>
				{:else if mcp.source === 'auto-detected'}
					<Badge variant="auto" title={mcp.sourcePath ? `Source: ${mcp.sourcePath}` : 'Auto-detected from filesystem'}>Auto</Badge>
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
						{(() => { try { return new URL(mcp.url).hostname; } catch { return mcp.url; } })()}
					</span>
				{/if}

				{#if showGatewayToggle && isInGateway}
					<Badge variant="warning" icon={Radio}>Gateway</Badge>
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
						aria-label={isInGateway ? `Remove ${mcp.name} from Gateway` : `Add ${mcp.name} to Gateway`}
					>
						<Radio class="w-4 h-4" aria-hidden="true" />
						{isInGateway ? 'In Gateway' : 'Add to Gateway'}
					</button>
				</div>
			{/if}
		</div>

		{#if showActions}
			<div class="flex items-center gap-1">
				{#if onFavoriteToggle}
					<FavoriteButton
						isFavorite={mcp.isFavorite}
						name={mcp.name}
						onclick={() => onFavoriteToggle(mcp, !mcp.isFavorite)}
					/>
				{/if}
				<ActionMenu bind:this={actionMenu} label="Actions for {mcp.name}">
					{#if onTest}
						<ActionMenuItem icon={Play} label="Test" onclick={() => { onTest(mcp); actionMenu.close(); }} />
					{/if}
					{#if onEdit && !isSystemMcp}
						<ActionMenuItem icon={Edit} label="Edit" onclick={() => { onEdit(mcp); actionMenu.close(); }} />
					{/if}
					{#if onDuplicate && !isSystemMcp}
						<ActionMenuItem icon={Copy} label="Duplicate" onclick={() => { onDuplicate(mcp); actionMenu.close(); }} />
					{/if}
					{#if onDelete && !isSystemMcp}
						<ActionMenuItem icon={Trash2} label="Delete" variant="danger" onclick={() => { onDelete(mcp); actionMenu.close(); }} />
					{/if}
				</ActionMenu>
			</div>
		{/if}
	</div>
</div>
