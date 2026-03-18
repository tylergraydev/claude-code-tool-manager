<script lang="ts">
	let open = false;
</script>

<div class="relative">
	<button class="p-1 hover:bg-muted rounded" on:click={() => (open = !open)}>
		⋮
	</button>
	{#if open}
		<div class="absolute right-0 mt-1 bg-popover border rounded-md shadow-lg z-50 min-w-[160px]" role="menu">
			<slot />
		</div>
	{/if}
</div>
