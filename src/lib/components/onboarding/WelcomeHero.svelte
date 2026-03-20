<script lang="ts">
	type Props = {
		projectCount: number;
		mcpCount: number;
		globalMcpCount: number;
	};

	let { projectCount, mcpCount, globalMcpCount }: Props = $props();

	const steps = $derived([
		{
			label: 'Add a project',
			done: projectCount > 0,
			description: 'Scan a project directory to manage its Claude configuration'
		},
		{
			label: 'Add an MCP server',
			done: mcpCount > 0 || globalMcpCount > 0,
			description: 'Connect an MCP server for extended capabilities'
		}
	]);
</script>

<div class="bg-gradient-to-br from-primary-50 to-primary-100 dark:from-primary-900/20 dark:to-primary-800/20 rounded-xl p-6 border border-primary-200 dark:border-primary-800">
	<h2 class="text-xl font-bold text-gray-900 dark:text-white mb-2">Getting Started</h2>
	<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
		Complete these steps to get the most out of Claude Code Tool Manager.
	</p>

	<div class="space-y-3">
		{#each steps as step}
			<div class="flex items-start gap-3 p-3 bg-white dark:bg-gray-800 rounded-lg">
				<div class="flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center {step.done ? 'bg-green-500 text-white' : 'bg-gray-200 dark:bg-gray-700'}">
					{#if step.done}
						&#10003;
					{:else}
						&bull;
					{/if}
				</div>
				<div>
					<p class="font-medium text-gray-900 dark:text-white">{step.label}</p>
					<p class="text-sm text-gray-500 dark:text-gray-400">{step.description}</p>
				</div>
			</div>
		{/each}
	</div>
</div>
