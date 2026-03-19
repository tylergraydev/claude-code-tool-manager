<script lang="ts">
	type Volume = {
		hostPath: string;
		containerPath: string;
		readOnly: boolean;
	};

	type Props = {
		volumes: Volume[];
	};

	let { volumes = [] }: Props = $props();

	function addVolume() {
		volumes = [...volumes, { hostPath: '', containerPath: '', readOnly: false }];
	}

	function removeVolume(index: number) {
		volumes = volumes.filter((_, i) => i !== index);
	}
</script>

<div>
	<div class="flex items-center justify-between mb-2">
		<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Volume Mappings</h4>
		<button type="button" onclick={addVolume} class="text-sm text-primary-600 hover:text-primary-700">
			Add Volume
		</button>
	</div>

	{#each volumes as volume, i}
		<div class="flex items-center gap-2 mb-2">
			<input type="text" bind:value={volume.hostPath} placeholder="Host path" class="flex-1 rounded border-gray-300 text-sm" />
			<span class="text-gray-400">:</span>
			<input type="text" bind:value={volume.containerPath} placeholder="Container path" class="flex-1 rounded border-gray-300 text-sm" />
			<label class="flex items-center gap-1 text-xs text-gray-500">
				<input type="checkbox" bind:checked={volume.readOnly} />
				RO
			</label>
			<button type="button" onclick={() => removeVolume(i)} class="text-red-500 text-sm">Remove</button>
		</div>
	{/each}
</div>
