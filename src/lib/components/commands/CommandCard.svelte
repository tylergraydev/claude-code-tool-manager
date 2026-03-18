<script lang="ts">
	import type { Command } from '$lib/types';
	import { Terminal, Edit, Trash2 } from 'lucide-svelte';
	import { ActionMenu, ActionMenuItem, FavoriteButton, Badge } from '$lib/components/shared';

	type Props = {
		command: Command;
		showActions?: boolean;
		onEdit?: (command: Command) => void;
		onDelete?: (command: Command) => void;
		onFavoriteToggle?: (command: Command, favorite: boolean) => void;
	};

	let {
		command,
		showActions = true,
		onEdit,
		onDelete,
		onFavoriteToggle
	}: Props = $props();

	let actionMenu: ActionMenu;
</script>

<div class="card group relative hover:shadow-md transition-all duration-200">
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl bg-amber-100 text-amber-600 dark:bg-amber-900/50 dark:text-amber-400 flex items-center justify-center">
			<Terminal class="w-5 h-5" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					/{command.name}
				</h3>
				{#if command.source === 'auto-detected'}
					<Badge variant="auto" title={command.sourcePath ? `Source: ${command.sourcePath}` : 'Auto-detected from filesystem'}>Auto</Badge>
				{/if}
			</div>

			{#if command.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{command.description}
				</p>
			{/if}

			<div class="flex items-center gap-1.5 mt-2 flex-wrap">
				{#if command.allowedTools && command.allowedTools.length > 0}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400">
						{command.allowedTools.length} tool{command.allowedTools.length !== 1 ? 's' : ''}
					</span>
				{/if}

				{#if command.argumentHint}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300 font-mono">
						{command.argumentHint}
					</span>
				{/if}

				{#if command.tags && command.tags.length > 0}
					{#each command.tags.slice(0, 2) as tag}
						<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
							{tag}
						</span>
					{/each}
					{#if command.tags.length > 2}
						<span class="text-xs text-gray-400">+{command.tags.length - 2}</span>
					{/if}
				{/if}
			</div>
		</div>

		{#if showActions}
			<div class="flex items-center gap-1">
				{#if onFavoriteToggle}
					<FavoriteButton
						isFavorite={command.isFavorite}
						name={command.name}
						onclick={() => onFavoriteToggle(command, !command.isFavorite)}
					/>
				{/if}
				<ActionMenu bind:this={actionMenu} label="Actions for {command.name}">
					{#if onEdit}
						<ActionMenuItem icon={Edit} label="Edit" onclick={() => { onEdit(command); actionMenu.close(); }} />
					{/if}
					{#if onDelete}
						<ActionMenuItem icon={Trash2} label="Delete" variant="danger" onclick={() => { onDelete(command); actionMenu.close(); }} />
					{/if}
				</ActionMenu>
			</div>
		{/if}
	</div>
</div>
