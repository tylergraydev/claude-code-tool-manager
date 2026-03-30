<script lang="ts">
	import { Play, Square, RotateCw, Trash2, Hammer } from 'lucide-svelte';
	import type { DockerStatusType } from '$lib/types';

	let { status, disabled = false, onBuild, onStart, onStop, onRestart, onRemove }: {
		status: DockerStatusType;
		disabled?: boolean;
		onBuild?: () => void;
		onStart?: () => void;
		onStop?: () => void;
		onRestart?: () => void;
		onRemove?: () => void;
	} = $props();

	const btnBase = "p-1.5 rounded-lg transition-colors disabled:opacity-50 disabled:pointer-events-none";
</script>

<div class="flex items-center gap-1">
	{#if status === 'not_created'}
		{#if onBuild}
			<button onclick={() => onBuild?.()} {disabled}
				class="{btnBase} text-gray-500 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20" title="Build">
				<Hammer class="w-4 h-4" aria-hidden="true" />
			</button>
		{/if}
		{#if onStart}
			<button onclick={() => onStart?.()} {disabled}
				class="{btnBase} text-gray-500 hover:text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20" title="Start">
				<Play class="w-4 h-4" aria-hidden="true" />
			</button>
		{/if}
	{:else if status === 'running'}
		{#if onStop}
			<button onclick={() => onStop?.()} {disabled}
				class="{btnBase} text-gray-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20" title="Stop">
				<Square class="w-4 h-4" aria-hidden="true" />
			</button>
		{/if}
		{#if onRestart}
			<button onclick={() => onRestart?.()} {disabled}
				class="{btnBase} text-gray-500 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/20" title="Restart">
				<RotateCw class="w-4 h-4" aria-hidden="true" />
			</button>
		{/if}
	{:else}
		{#if onStart}
			<button onclick={() => onStart?.()} {disabled}
				class="{btnBase} text-gray-500 hover:text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20" title="Start">
				<Play class="w-4 h-4" aria-hidden="true" />
			</button>
		{/if}
	{/if}
	{#if status !== 'not_created' && status !== 'running' && onRemove}
		<button onclick={() => onRemove?.()} {disabled}
			class="{btnBase} text-gray-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20" title="Remove Docker container">
			<Trash2 class="w-4 h-4" aria-hidden="true" />
		</button>
	{/if}
</div>
