<script lang="ts">
	import type { Skill } from '$lib/types';
	import { Terminal, Sparkles, MoreVertical, Edit, Trash2 } from 'lucide-svelte';

	type Props = {
		skill: Skill;
		showActions?: boolean;
		onEdit?: (skill: Skill) => void;
		onDelete?: (skill: Skill) => void;
	};

	let {
		skill,
		showActions = true,
		onEdit,
		onDelete
	}: Props = $props();

	let showMenu = $state(false);

	function closeMenu() {
		showMenu = false;
	}

	const isCommand = skill.skillType === 'command' || !skill.skillType;
</script>

<svelte:window onclick={closeMenu} />

<div class="card group relative hover:shadow-md transition-all duration-200">
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl {isCommand ? 'bg-amber-100 text-amber-600 dark:bg-amber-900/50 dark:text-amber-400' : 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400'} flex items-center justify-center">
			{#if isCommand}
				<Terminal class="w-5 h-5" />
			{:else}
				<Sparkles class="w-5 h-5" />
			{/if}
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{#if isCommand}
						/{skill.name}
					{:else}
						{skill.name}
					{/if}
				</h3>
				<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium {isCommand ? 'bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300' : 'bg-purple-100 text-purple-700 dark:bg-purple-900/50 dark:text-purple-300'}">
					{isCommand ? 'Command' : 'Skill'}
				</span>
				{#if skill.source === 'auto-detected'}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300">
						Auto
					</span>
				{/if}
			</div>

			{#if skill.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{skill.description}
				</p>
			{/if}

			<div class="flex items-center gap-1.5 mt-2 flex-wrap">
				{#if skill.allowedTools && skill.allowedTools.length > 0}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400">
						{skill.allowedTools.length} tool{skill.allowedTools.length !== 1 ? 's' : ''}
					</span>
				{/if}

				{#if isCommand && skill.argumentHint}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300 font-mono">
						{skill.argumentHint}
					</span>
				{/if}

				{#if skill.tags && skill.tags.length > 0}
					{#each skill.tags.slice(0, 2) as tag}
						<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
							{tag}
						</span>
					{/each}
					{#if skill.tags.length > 2}
						<span class="text-xs text-gray-400">+{skill.tags.length - 2}</span>
					{/if}
				{/if}
			</div>
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
						{#if onEdit}
							<button
								onclick={() => {
									onEdit(skill);
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
									onDelete(skill);
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
