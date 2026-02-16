<script lang="ts">
	import { PERMISSION_DEFAULT_MODES } from '$lib/types';

	type Props = {
		value: string | undefined;
		onchange: (mode: string | null) => void;
	};

	let { value, onchange }: Props = $props();

	function handleChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		onchange(target.value || null);
	}
</script>

<div class="flex items-center gap-3">
	<label class="text-sm font-medium text-gray-700 dark:text-gray-300 whitespace-nowrap">
		Default Mode
	</label>
	<select value={value ?? ''} onchange={handleChange} class="input text-sm flex-1 max-w-xs">
		{#each PERMISSION_DEFAULT_MODES as mode}
			<option value={mode.value}>{mode.label}</option>
		{/each}
	</select>
	{#if value}
		<span class="text-xs text-gray-500 dark:text-gray-400">
			{PERMISSION_DEFAULT_MODES.find((m) => m.value === value)?.description ?? ''}
		</span>
	{/if}
</div>
