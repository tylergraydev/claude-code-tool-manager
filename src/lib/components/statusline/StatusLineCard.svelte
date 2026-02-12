<script lang="ts">
	import type { StatusLine } from '$lib/types';
	import { PanelBottom, Play, Edit, Trash2, MoreVertical, X, ExternalLink } from 'lucide-svelte';

	type Props = {
		statusline: StatusLine;
		onActivate?: (sl: StatusLine) => void;
		onDeactivate?: () => void;
		onEdit?: (sl: StatusLine) => void;
		onDelete?: (sl: StatusLine) => void;
	};

	let { statusline, onActivate, onDeactivate, onEdit, onDelete }: Props = $props();

	let showMenu = $state(false);

	const typeBadgeClass: Record<string, string> = {
		custom: 'bg-purple-100 text-purple-700 dark:bg-purple-900/50 dark:text-purple-400',
		premade: 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-400',
		raw: 'bg-gray-100 text-gray-700 dark:bg-gray-900/50 dark:text-gray-400'
	};

	const typeLabel: Record<string, string> = {
		custom: 'Custom',
		premade: 'Premade',
		raw: 'Raw'
	};
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border transition-all hover:shadow-md
		{statusline.isActive
			? 'border-green-300 dark:border-green-600 ring-1 ring-green-200 dark:ring-green-700'
			: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
>
	<div class="p-4">
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-3 min-w-0">
				<div
					class="w-10 h-10 rounded-lg flex items-center justify-center text-lg shrink-0
						{statusline.isActive
							? 'bg-green-100 dark:bg-green-900/50'
							: 'bg-gray-100 dark:bg-gray-700'}"
				>
					<PanelBottom class="w-5 h-5 {statusline.isActive ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}" />
				</div>
				<div class="min-w-0">
					<div class="flex items-center gap-2">
						<h3 class="font-medium text-gray-900 dark:text-white truncate">{statusline.name}</h3>
						<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {typeBadgeClass[statusline.statuslineType] || ''}">
							{typeLabel[statusline.statuslineType] || statusline.statuslineType}
						</span>
						{#if statusline.isActive}
							<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400">
								Active
							</span>
						{/if}
					</div>
					{#if statusline.description}
						<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">
							{statusline.description}
						</p>
					{/if}
					{#if statusline.author}
						<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
							by {statusline.author}
						</p>
					{/if}
				</div>
			</div>

			<div class="relative shrink-0 ml-2">
				<button
					onclick={(e) => { e.stopPropagation(); showMenu = !showMenu; }}
					class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
				>
					<MoreVertical class="w-4 h-4" />
				</button>

				{#if showMenu}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="fixed inset-0 z-40" onclick={() => (showMenu = false)}></div>
					<div class="absolute right-0 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50">
						<button
							onclick={() => { showMenu = false; onEdit?.(statusline); }}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
						>
							<Edit class="w-4 h-4" />
							Edit
						</button>
						{#if statusline.homepageUrl}
							<a
								href={statusline.homepageUrl}
								target="_blank"
								rel="noopener noreferrer"
								class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
								onclick={() => (showMenu = false)}
							>
								<ExternalLink class="w-4 h-4" />
								Homepage
							</a>
						{/if}
						<hr class="my-1 border-gray-200 dark:border-gray-700" />
						<button
							onclick={() => { showMenu = false; onDelete?.(statusline); }}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
						>
							<Trash2 class="w-4 h-4" />
							Delete
						</button>
					</div>
				{/if}
			</div>
		</div>

		<div class="mt-4 flex gap-2">
			{#if statusline.isActive}
				<button
					onclick={() => onDeactivate?.()}
					class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
				>
					<X class="w-3.5 h-3.5" />
					Deactivate
				</button>
			{:else}
				<button
					onclick={() => onActivate?.(statusline)}
					class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition-colors"
				>
					<Play class="w-3.5 h-3.5" />
					Activate
				</button>
			{/if}
		</div>
	</div>
</div>
