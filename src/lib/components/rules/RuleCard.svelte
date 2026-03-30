<script lang="ts">
	import type { Rule } from '$lib/types';
	import { BookOpen, MoreVertical, Edit, Trash2, Heart, Link } from 'lucide-svelte';

	type Props = {
		rule: Rule;
		showActions?: boolean;
		onEdit?: (rule: Rule) => void;
		onDelete?: (rule: Rule) => void;
		onFavoriteToggle?: (rule: Rule, favorite: boolean) => void;
	};

	let {
		rule,
		showActions = true,
		onEdit,
		onDelete,
		onFavoriteToggle
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
			{#if rule.isSymlink}
				<Link class="w-5 h-5" />
			{:else}
				<BookOpen class="w-5 h-5" />
			{/if}
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{rule.name}
				</h3>
				{#if rule.source === 'auto-detected'}
					<span
						class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300 cursor-help"
						title={rule.sourcePath ? `Source: ${rule.sourcePath}` : 'Auto-detected from filesystem'}
					>
						Auto
					</span>
				{/if}
				{#if rule.isSymlink}
					<span
						class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300 cursor-help"
						title={rule.symlinkTarget ? `Links to: ${rule.symlinkTarget}` : 'Symlinked rule'}
					>
						Symlink
					</span>
				{/if}
			</div>

			{#if rule.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{rule.description}
				</p>
			{/if}

			<div class="flex items-center gap-1.5 mt-2 flex-wrap">
				{#if rule.paths && rule.paths.length > 0}
					{#each rule.paths.slice(0, 3) as path}
						<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-amber-100 text-amber-600 dark:bg-amber-900/50 dark:text-amber-400 font-mono">
							{path}
						</span>
					{/each}
					{#if rule.paths.length > 3}
						<span class="text-xs text-gray-400">+{rule.paths.length - 3}</span>
					{/if}
				{:else}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-600 dark:bg-green-900/50 dark:text-green-400">
						Always active
					</span>
				{/if}

				{#if rule.tags && rule.tags.length > 0}
					{#each rule.tags.slice(0, 2) as tag}
						<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
							{tag}
						</span>
					{/each}
					{#if rule.tags.length > 2}
						<span class="text-xs text-gray-400">+{rule.tags.length - 2}</span>
					{/if}
				{/if}
			</div>
		</div>

		{#if showActions}
			<div class="flex items-center gap-1">
				{#if onFavoriteToggle}
					<button
						onclick={(e) => {
							e.stopPropagation();
							onFavoriteToggle(rule, !rule.isFavorite);
						}}
						class="p-1.5 rounded-lg transition-colors {rule.isFavorite
							? 'text-rose-500 hover:text-rose-600'
							: 'text-gray-300 hover:text-rose-400 dark:text-gray-600 dark:hover:text-rose-400'}"
						title={rule.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
					>
						<Heart class="w-4 h-4" fill={rule.isFavorite ? 'currentColor' : 'none'} />
					</button>
				{/if}
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
										onEdit(rule);
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
										onDelete(rule);
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
			</div>
		{/if}
	</div>
</div>
