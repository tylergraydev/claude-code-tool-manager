<script lang="ts">
	import { Play, Square, RotateCw, Hammer, Trash2 } from 'lucide-svelte';

	type Props = {
		status: string;
		onBuild?: () => void;
		onStart?: () => void;
		onStop?: () => void;
		onRestart?: () => void;
		onRemove?: () => void;
	};

	let { status, onBuild, onStart, onStop, onRestart, onRemove }: Props = $props();

	const canRemove = $derived(status !== 'running' && status !== 'not_created');
</script>

<div class="flex items-center gap-1">
	{#if status === 'not_created'}
		{#if onBuild}
			<button title="Build" onclick={onBuild} class="btn btn-ghost text-sm px-2 py-1.5">
				<Hammer class="w-3.5 h-3.5 mr-1" />
				Build
			</button>
		{/if}
		{#if onStart}
			<button title="Start" onclick={onStart} class="btn btn-ghost text-sm px-2 py-1.5 text-green-600 dark:text-green-400">
				<Play class="w-3.5 h-3.5 mr-1" />
				Start
			</button>
		{/if}
	{:else if status === 'running'}
		{#if onStop}
			<button title="Stop" onclick={onStop} class="btn btn-ghost text-sm px-2 py-1.5 text-red-600 dark:text-red-400">
				<Square class="w-3.5 h-3.5 mr-1" />
				Stop
			</button>
		{/if}
		{#if onRestart}
			<button title="Restart" onclick={onRestart} class="btn btn-ghost text-sm px-2 py-1.5">
				<RotateCw class="w-3.5 h-3.5 mr-1" />
				Restart
			</button>
		{/if}
	{:else}
		{#if onStart}
			<button title="Start" onclick={onStart} class="btn btn-ghost text-sm px-2 py-1.5 text-green-600 dark:text-green-400">
				<Play class="w-3.5 h-3.5 mr-1" />
				Start
			</button>
		{/if}
	{/if}

	{#if canRemove && onRemove}
		<button title="Remove Docker container" onclick={onRemove} class="btn btn-ghost text-sm px-2 py-1.5 text-red-500 hover:text-red-700">
			<Trash2 class="w-3.5 h-3.5 mr-1" />
			Remove
		</button>
	{/if}
</div>
