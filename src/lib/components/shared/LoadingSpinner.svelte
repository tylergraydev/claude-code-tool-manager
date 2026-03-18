<script lang="ts">
	export let size: string = 'md';
</script>

<div class="flex items-center justify-center" class:text-sm={size === 'sm'} class:text-base={size === 'md'} class:text-lg={size === 'lg'}>
	<div class="animate-spin rounded-full border-2 border-muted-foreground border-t-transparent h-5 w-5"></div>
</div>
