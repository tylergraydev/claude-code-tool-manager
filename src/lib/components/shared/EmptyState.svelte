<script lang="ts">
	export let message: string = 'No items found';
	export let icon: string = '';
</script>

<div class="flex flex-col items-center justify-center py-12 text-center">
	{#if icon}
		<span class="text-4xl mb-4">{icon}</span>
	{/if}
	<p class="text-muted-foreground">{message}</p>
	<slot />
</div>
