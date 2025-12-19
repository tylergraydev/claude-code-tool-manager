<script lang="ts">
	import { Search, X } from 'lucide-svelte';

	type Props = {
		value: string;
		placeholder?: string;
		onchange?: (value: string) => void;
	};

	let { value = $bindable(''), placeholder = 'Search...', onchange }: Props = $props();

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		value = target.value;
		onchange?.(value);
	}

	function clear() {
		value = '';
		onchange?.('');
	}
</script>

<div class="relative">
	<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
	<input
		type="text"
		{value}
		oninput={handleInput}
		{placeholder}
		class="input pl-10 pr-10"
	/>
	{#if value}
		<button
			onclick={clear}
			class="absolute right-3 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
		>
			<X class="w-4 h-4" />
		</button>
	{/if}
</div>
