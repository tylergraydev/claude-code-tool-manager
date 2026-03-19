<script lang="ts">
	type Props = {
		status: string;
	};

	let { status }: Props = $props();

	const labels: Record<string, string> = {
		running: 'Running',
		stopped: 'Stopped',
		exited: 'Exited',
		created: 'Created',
		not_created: 'Not Created',
		unknown: 'Unknown'
	};

	const colors: Record<string, string> = {
		running: 'text-green-600 bg-green-100 dark:bg-green-900/50 dark:text-green-400',
		stopped: 'text-gray-600 bg-gray-100 dark:bg-gray-900/50 dark:text-gray-400',
		exited: 'text-red-600 bg-red-100 dark:bg-red-900/50 dark:text-red-400',
		created: 'text-blue-600 bg-blue-100 dark:bg-blue-900/50 dark:text-blue-400',
		not_created: 'text-yellow-600 bg-yellow-100 dark:bg-yellow-900/50 dark:text-yellow-400',
		unknown: 'text-gray-600 bg-gray-100 dark:bg-gray-900/50 dark:text-gray-400'
	};
</script>

<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {colors[status] || colors.unknown}">
	{labels[status] || 'Unknown'}
</span>
