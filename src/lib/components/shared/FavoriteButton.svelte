<script lang="ts">
	import { Star } from 'lucide-svelte';

	type Props = {
		isFavorite: boolean;
		name?: string;
		onclick?: () => void;
	};

	let { isFavorite, name = '', onclick }: Props = $props();
</script>

<button
	class="p-1.5 rounded-lg transition-colors {isFavorite
		? 'text-yellow-500 hover:text-yellow-600'
		: 'text-gray-400 hover:text-yellow-500'}"
	onclick={(e) => { e.stopPropagation(); onclick?.(); }}
	aria-label={isFavorite ? `Remove ${name} from favorites` : `Add ${name} to favorites`}
>
	<Star class="w-4 h-4" fill={isFavorite ? 'currentColor' : 'none'} />
</button>
