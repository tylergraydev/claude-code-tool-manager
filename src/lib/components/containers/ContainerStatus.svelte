<script lang="ts">
	import type { DockerStatusType } from '$lib/types';

	let { status }: { status: DockerStatusType } = $props();

	const statusConfig: Record<DockerStatusType, { color: string; label: string }> = {
		running: { color: 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400', label: 'Running' },
		stopped: { color: 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400', label: 'Stopped' },
		exited: { color: 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400', label: 'Exited' },
		created: { color: 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-400', label: 'Created' },
		not_created: { color: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-500', label: 'Not Created' },
		unknown: { color: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-400', label: 'Unknown' }
	};

	const config = $derived(statusConfig[status] || statusConfig.unknown);
</script>

<span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium {config.color}">
	<span class="w-1.5 h-1.5 rounded-full {status === 'running' ? 'bg-green-500 animate-pulse' : status === 'exited' ? 'bg-red-500' : 'bg-current opacity-40'}" aria-hidden="true"></span>
	{config.label}
</span>
