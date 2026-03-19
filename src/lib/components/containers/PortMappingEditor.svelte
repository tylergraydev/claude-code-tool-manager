<script lang="ts">
	type Port = {
		hostPort: number;
		containerPort: number;
		protocol: string;
	};

	type Props = {
		ports: Port[];
	};

	let { ports = [] }: Props = $props();

	function addPort() {
		ports = [...ports, { hostPort: 0, containerPort: 0, protocol: 'tcp' }];
	}

	function removePort(index: number) {
		ports = ports.filter((_, i) => i !== index);
	}
</script>

<div>
	<div class="flex items-center justify-between mb-2">
		<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Port Mappings</h4>
		<button type="button" onclick={addPort} class="text-sm text-primary-600 hover:text-primary-700">
			Add Port
		</button>
	</div>

	{#each ports as port, i}
		<div class="flex items-center gap-2 mb-2">
			<input type="number" bind:value={port.hostPort} placeholder="Host" class="w-24 rounded border-gray-300 text-sm" />
			<span class="text-gray-400">:</span>
			<input type="number" bind:value={port.containerPort} placeholder="Container" class="w-24 rounded border-gray-300 text-sm" />
			<span class="text-xs text-gray-500 uppercase">{port.protocol === 'tcp' ? 'TCP' : 'UDP'}</span>
			<button type="button" onclick={() => removePort(i)} class="text-red-500 text-sm">Remove</button>
		</div>
	{/each}
</div>
