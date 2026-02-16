<script lang="ts">
	import type { MemoryFileInfo } from '$lib/types';
	import { Copy, Check } from 'lucide-svelte';

	type Props = {
		file: MemoryFileInfo;
	};

	let { file }: Props = $props();
	let copied = $state(false);

	function formatRelativeTime(isoString: string): string {
		const date = new Date(isoString);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		const diffHours = Math.floor(diffMins / 60);
		const diffDays = Math.floor(diffHours / 24);

		if (diffMins < 1) return 'just now';
		if (diffMins < 60) return `${diffMins}m ago`;
		if (diffHours < 24) return `${diffHours}h ago`;
		if (diffDays < 30) return `${diffDays}d ago`;
		return date.toLocaleDateString();
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	async function copyPath() {
		try {
			await navigator.clipboard.writeText(file.filePath);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Clipboard API not available
		}
	}
</script>

<div class="flex flex-wrap items-center gap-x-4 gap-y-1 px-4 py-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg text-sm">
	<!-- File path -->
	<div class="flex items-center gap-1.5 text-gray-600 dark:text-gray-300 min-w-0">
		<span class="truncate font-mono text-xs">{file.filePath}</span>
		<button
			onclick={copyPath}
			class="flex-shrink-0 p-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
			title="Copy path"
		>
			{#if copied}
				<Check class="w-3.5 h-3.5 text-green-500" />
			{:else}
				<Copy class="w-3.5 h-3.5 text-gray-400" />
			{/if}
		</button>
	</div>

	<!-- Exists badge -->
	<span
		class="px-2 py-0.5 rounded-full text-xs font-medium
			{file.exists
			? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
			: 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400'}"
	>
		{file.exists ? 'Exists' : 'Not created'}
	</span>

	<!-- Last modified -->
	{#if file.lastModified}
		<span class="text-gray-400 dark:text-gray-500 text-xs">
			Modified: {formatRelativeTime(file.lastModified)}
		</span>
	{/if}

	<!-- Size -->
	{#if file.sizeBytes !== undefined && file.sizeBytes !== null}
		<span class="text-gray-400 dark:text-gray-500 text-xs">
			{formatSize(file.sizeBytes)}
		</span>
	{/if}
</div>
