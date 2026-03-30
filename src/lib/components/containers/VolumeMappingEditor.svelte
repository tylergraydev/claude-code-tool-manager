<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import type { VolumeMapping } from '$lib/types';

	let { volumes = $bindable<VolumeMapping[]>([]) }: { volumes: VolumeMapping[] } = $props();

	function addVolume() {
		volumes = [...volumes, { hostPath: '', containerPath: '', readOnly: false }];
	}

	function removeVolume(index: number) {
		volumes = volumes.filter((_, i) => i !== index);
	}
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<label class="text-sm font-medium text-gray-700 dark:text-gray-300">Volume Mappings</label>
		<button type="button" onclick={addVolume}
			class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1">
			<Plus class="w-3 h-3" aria-hidden="true" /> Add Volume
		</button>
	</div>
	{#if volumes.length > 0}
		<div class="space-y-2">
			{#each volumes as vol, i}
				<div class="flex items-center gap-2">
					<input type="text" bind:value={vol.hostPath} placeholder="Host path"
						class="input flex-1 py-1.5" />
					<span class="text-gray-400">:</span>
					<input type="text" bind:value={vol.containerPath} placeholder="Container path"
						class="input flex-1 py-1.5" />
					<label class="flex items-center gap-1 text-xs text-gray-500">
						<input type="checkbox" bind:checked={vol.readOnly} class="rounded" />
						RO
					</label>
					<button type="button" onclick={() => removeVolume(i)} class="p-1 text-gray-400 hover:text-red-500">
						<Trash2 class="w-4 h-4" aria-hidden="true" />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>
