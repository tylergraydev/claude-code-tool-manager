<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import type { PortMapping } from '$lib/types';

	let { ports = $bindable<PortMapping[]>([]) }: { ports: PortMapping[] } = $props();

	function addPort() {
		ports = [...ports, { hostPort: 0, containerPort: 0, protocol: 'tcp' }];
	}

	function removePort(index: number) {
		ports = ports.filter((_, i) => i !== index);
	}
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<label class="text-sm font-medium text-gray-700 dark:text-gray-300">Port Mappings</label>
		<button type="button" onclick={addPort}
			class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1">
			<Plus class="w-3 h-3" aria-hidden="true" /> Add Port
		</button>
	</div>
	{#if ports.length > 0}
		<div class="space-y-2">
			<div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
				<span class="flex-1">Host Port</span>
				<span class="w-3"></span>
				<span class="flex-1">Container Port</span>
				<span class="w-[4.5rem]"></span>
				<span class="w-8"></span>
			</div>
			{#each ports as port, i}
				<div class="flex items-center gap-2">
					<input type="number" bind:value={port.hostPort} placeholder="e.g. 3000"
						class="input flex-1 py-1.5" />
					<span class="text-gray-400">:</span>
					<input type="number" bind:value={port.containerPort} placeholder="e.g. 3000"
						class="input flex-1 py-1.5" />
					<select bind:value={port.protocol}
						class="input w-auto py-1.5">
						<option value="tcp">TCP</option>
						<option value="udp">UDP</option>
					</select>
					<button type="button" onclick={() => removePort(i)} class="p-1 text-gray-400 hover:text-red-500">
						<Trash2 class="w-4 h-4" aria-hidden="true" />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>
