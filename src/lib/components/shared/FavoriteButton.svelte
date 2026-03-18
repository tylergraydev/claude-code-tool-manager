<script lang="ts">
	export let isFavorite: boolean = false;
	export let onClick: () => void = () => {};
</script>

<button
	class="p-1 hover:text-yellow-500 transition-colors"
	class:text-yellow-500={isFavorite}
	on:click|stopPropagation={onClick}
	title={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
>
	{isFavorite ? '★' : '☆'}
</button>
