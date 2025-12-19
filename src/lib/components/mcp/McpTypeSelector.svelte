<script lang="ts">
	import type { McpType } from '$lib/types';
	import { Plug, Globe, Server } from 'lucide-svelte';

	type Props = {
		value: McpType;
	};

	let { value = $bindable('stdio') }: Props = $props();

	const types: { value: McpType; label: string; icon: typeof Plug; description: string }[] = [
		{
			value: 'stdio',
			label: 'Standard I/O',
			icon: Plug,
			description: 'Local command-line tool (npx, python, etc.)'
		},
		{
			value: 'sse',
			label: 'Server-Sent Events',
			icon: Globe,
			description: 'Cloud service with SSE endpoint'
		},
		{
			value: 'http',
			label: 'HTTP/REST',
			icon: Server,
			description: 'REST API with token authentication'
		}
	];
</script>

<div class="space-y-2">
	<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
		Connection Type <span class="text-red-500">*</span>
	</label>

	<div class="grid grid-cols-3 gap-3">
		{#each types as type}
			<button
				type="button"
				onclick={() => (value = type.value)}
				class="flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all
					{value === type.value
						? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
						: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div
					class="w-10 h-10 rounded-lg flex items-center justify-center
					{value === type.value
						? 'bg-primary-100 text-primary-600 dark:bg-primary-800 dark:text-primary-300'
						: 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'}"
				>
					<svelte:component this={type.icon} class="w-5 h-5" />
				</div>
				<span
					class="text-sm font-medium
					{value === type.value
						? 'text-primary-700 dark:text-primary-300'
						: 'text-gray-700 dark:text-gray-300'}"
				>
					{type.label}
				</span>
				<span class="text-[10px] text-gray-500 dark:text-gray-400 text-center leading-tight">
					{type.description}
				</span>
			</button>
		{/each}
	</div>
</div>
