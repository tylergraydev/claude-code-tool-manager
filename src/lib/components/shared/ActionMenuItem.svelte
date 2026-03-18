<script lang="ts">
	export let onClick: () => void = () => {};
	export let variant: 'default' | 'danger' = 'default';
</script>

<button
	class="w-full text-left px-3 py-2 text-sm hover:bg-muted transition-colors"
	class:text-red-400={variant === 'danger'}
	on:click={onClick}
	role="menuitem"
>
	<slot />
</button>
