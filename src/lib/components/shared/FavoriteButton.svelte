<script lang="ts">
	import { Star } from 'lucide-svelte';

	type Props = {
		isFavorite?: boolean;
		name?: string;
		onclick?: () => void;
	};

	let { isFavorite = false, name = '', onclick }: Props = $props();

	const label = $derived(
		isFavorite
			? name ? `Remove ${name} from favorites` : 'Remove from favorites'
			: name ? `Add ${name} to favorites` : 'Add to favorites'
	);
</script>

<button
	class="p-1 hover:text-yellow-500 transition-colors"
	class:text-yellow-500={isFavorite}
	{onclick}
	title={label}
	aria-label={label}
>
	<Star class="w-4 h-4" fill={isFavorite ? 'currentColor' : 'none'} />
</button>
