<script lang="ts">
	type Props = {
		size?: 'sm' | 'md' | 'lg';
	};

	let { size = 'md' }: Props = $props();

	const sizeClasses: Record<string, string> = {
		sm: 'h-5 w-5',
		md: 'h-8 w-8',
		lg: 'h-12 w-12'
	};
</script>

<div class="flex items-center justify-center" role="status" aria-label="Loading">
	<div class="animate-spin rounded-full border-2 border-gray-300 border-t-blue-600 {sizeClasses[size]}"></div>
</div>
