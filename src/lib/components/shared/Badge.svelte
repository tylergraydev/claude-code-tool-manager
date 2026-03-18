<script lang="ts">
	export let variant: 'default' | 'success' | 'warning' | 'error' = 'default';

	const variantClasses: Record<string, string> = {
		default: 'bg-muted',
		success: 'bg-green-500/20 text-green-400',
		warning: 'bg-yellow-500/20 text-yellow-400',
		error: 'bg-red-500/20 text-red-400'
	};
</script>

<span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {variantClasses[variant] ?? ''}">
	<slot />
</span>
