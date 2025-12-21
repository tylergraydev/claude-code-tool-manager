<script lang="ts">
	import type { Hook } from '$lib/types';
	import { Zap, Terminal, MessageSquare, MoreVertical, Edit, Trash2, Copy } from 'lucide-svelte';

	type Props = {
		hook: Hook;
		showActions?: boolean;
		onEdit?: (hook: Hook) => void;
		onDelete?: (hook: Hook) => void;
		onDuplicate?: (hook: Hook) => void;
	};

	let {
		hook,
		showActions = true,
		onEdit,
		onDelete,
		onDuplicate
	}: Props = $props();

	let showMenu = $state(false);

	function closeMenu() {
		showMenu = false;
	}

	const isCommand = hook.hookType === 'command';

	// Event type color mapping
	const eventColors: Record<string, string> = {
		PreToolUse: 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300',
		PostToolUse: 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300',
		Notification: 'bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300',
		Stop: 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300',
		SubagentStop: 'bg-purple-100 text-purple-700 dark:bg-purple-900/50 dark:text-purple-300'
	};

	const eventColor = eventColors[hook.eventType] || 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300';
</script>

<svelte:window onclick={closeMenu} />

<div class="card group relative hover:shadow-md transition-all duration-200">
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl bg-orange-100 text-orange-600 dark:bg-orange-900/50 dark:text-orange-400 flex items-center justify-center">
			<Zap class="w-5 h-5" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 flex-wrap">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{hook.name}
				</h3>
				<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium {eventColor}">
					{hook.eventType}
				</span>
				{#if hook.isTemplate}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300">
						Template
					</span>
				{/if}
			</div>

			{#if hook.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{hook.description}
				</p>
			{/if}

			<div class="flex items-center gap-1.5 mt-2 flex-wrap">
				<!-- Hook type badge -->
				<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium {isCommand ? 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300' : 'bg-violet-100 text-violet-600 dark:bg-violet-900/50 dark:text-violet-300'}">
					{#if isCommand}
						<Terminal class="w-3 h-3" />
						Command
					{:else}
						<MessageSquare class="w-3 h-3" />
						Prompt
					{/if}
				</span>

				<!-- Matcher badge -->
				{#if hook.matcher}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400 font-mono">
						{hook.matcher}
					</span>
				{/if}

				<!-- Timeout badge -->
				{#if hook.timeout}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
						{hook.timeout}s timeout
					</span>
				{/if}

				<!-- Tags -->
				{#if hook.tags && hook.tags.length > 0}
					{#each hook.tags.slice(0, 2) as tag}
						<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
							{tag}
						</span>
					{/each}
					{#if hook.tags.length > 2}
						<span class="text-xs text-gray-400">+{hook.tags.length - 2}</span>
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
									onEdit(hook);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Edit class="w-4 h-4" />
								Edit
							</button>
						{/if}
						{#if onDuplicate}
							<button
								onclick={() => {
									onDuplicate(hook);
									closeMenu();
								}}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
							>
								<Copy class="w-4 h-4" />
								Duplicate
							</button>
						{/if}
						{#if onDelete}
							<button
								onclick={() => {
									onDelete(hook);
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
