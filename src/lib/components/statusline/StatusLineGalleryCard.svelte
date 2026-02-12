<script lang="ts">
	import type { StatusLineGalleryEntry } from '$lib/types';
	import { Download, ExternalLink, Package } from 'lucide-svelte';

	type Props = {
		entry: StatusLineGalleryEntry;
		isInstalled?: boolean;
		isInstalling?: boolean;
		onInstall?: (entry: StatusLineGalleryEntry) => void;
	};

	let { entry, isInstalled = false, isInstalling = false, onInstall }: Props = $props();
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600 transition-all hover:shadow-md">
	<div class="p-4">
		<div class="flex items-start gap-3">
			<div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/50 flex items-center justify-center shrink-0">
				<Package class="w-5 h-5 text-blue-600 dark:text-blue-400" />
			</div>
			<div class="min-w-0 flex-1">
				<div class="flex items-center gap-2">
					<h3 class="font-medium text-gray-900 dark:text-white truncate">{entry.name}</h3>
					{#if entry.homepageUrl}
						<a
							href={entry.homepageUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-gray-400 hover:text-primary-500 shrink-0"
						>
							<ExternalLink class="w-3.5 h-3.5" />
						</a>
					{/if}
				</div>
				{#if entry.description}
					<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">
						{entry.description}
					</p>
				{/if}
				{#if entry.author}
					<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">by {entry.author}</p>
				{/if}
			</div>
		</div>

		<!-- Preview -->
		{#if entry.previewText}
			<div class="mt-3 bg-gray-900 rounded-lg px-3 py-2 font-mono text-xs text-gray-300 overflow-x-auto">
				{entry.previewText}
			</div>
		{/if}

		<!-- Tags -->
		{#if entry.tags && entry.tags.length > 0}
			<div class="mt-3 flex flex-wrap gap-1">
				{#each entry.tags as tag}
					<span class="inline-flex items-center px-2 py-0.5 rounded text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
						{tag}
					</span>
				{/each}
			</div>
		{/if}

		<!-- Install button -->
		<div class="mt-4">
			{#if isInstalled}
				<button
					disabled
					class="w-full flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg border border-green-300 dark:border-green-600 text-green-700 dark:text-green-400 bg-green-50 dark:bg-green-900/20"
				>
					Installed
				</button>
			{:else}
				<button
					onclick={() => onInstall?.(entry)}
					disabled={isInstalling}
					class="w-full flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition-colors disabled:opacity-50"
				>
					<Download class="w-3.5 h-3.5" />
					{isInstalling ? 'Installing...' : 'Add to Library'}
				</button>
			{/if}
		</div>
	</div>
</div>
