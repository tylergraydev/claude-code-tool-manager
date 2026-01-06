<script lang="ts">
	import type { Command } from '$lib/types';
	import { Terminal, MoreVertical, Edit, Trash2 } from 'lucide-svelte';

	type Props = {
		command: Command;
		showActions?: boolean;
		onEdit?: (command: Command) => void;
		onDelete?: (command: Command) => void;
	};

	let {
		command,
		showActions = true,
		onEdit,
		onDelete
	}: Props = $props();

	let showMenu = $state(false);
	let menuAbove = $state(false);
	let menuButton: HTMLButtonElement;

	function closeMenu() {
		showMenu = false;
	}

	function toggleMenu(e: MouseEvent) {
		e.stopPropagation();
		if (!showMenu) {
			const rect = menuButton.getBoundingClientRect();
			const spaceBelow = window.innerHeight - rect.bottom;
			const menuHeight = 120;
			menuAbove = spaceBelow < menuHeight;
		}
		showMenu = !showMenu;
	}
</script>

<svelte:window onclick={closeMenu} />

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
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300">
						Auto
					</span>
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
			<div class="relative">
				<button
					bind:this={menuButton}
					onclick={toggleMenu}
					class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
				>
					<MoreVertical class="w-4 h-4" />
				</button>

				{#if showMenu}
					<div
						class="absolute right-0 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-10
							{menuAbove ? 'bottom-full mb-1' : 'top-full mt-1'}"
						onclick={(e) => e.stopPropagation()}
					>
						{#if onEdit}
							<button
								onclick={() => {
									onEdit(command);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Edit class="w-4 h-4" />
								Edit
							</button>
						{/if}
						{#if onDelete}
							<button
								onclick={() => {
									onDelete(command);
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
