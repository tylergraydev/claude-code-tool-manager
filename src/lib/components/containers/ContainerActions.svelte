<script lang="ts">
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
			<button title="Build" onclick={onBuild} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Build
			</button>
		{/if}
		{#if onStart}
			<button title="Start" onclick={onStart} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Start
			</button>
		{/if}
	{:else if status === 'running'}
		{#if onStop}
			<button title="Stop" onclick={onStop} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Stop
			</button>
		{/if}
		{#if onRestart}
			<button title="Restart" onclick={onRestart} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Restart
			</button>
		{/if}
	{:else}
		{#if onStart}
			<button title="Start" onclick={onStart} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Start
			</button>
		{/if}
	{/if}

	{#if canRemove && onRemove}
		<button title="Remove Docker container" onclick={onRemove} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-red-500">
			Remove
		</button>
	{/if}
</div>
