<script lang="ts">
	import { onMount } from 'svelte';
	import { soundLibrary } from '$lib/stores';
	import type { SystemSound } from '$lib/types';
	import { Volume2, Play, Pause, ChevronDown, FolderOpen } from 'lucide-svelte';
	import SoundBrowser from './SoundBrowser.svelte';

	type Props = {
		value?: string;
		onchange?: (path: string) => void;
		placeholder?: string;
	};

	let { value = '', onchange, placeholder = 'Select a sound...' }: Props = $props();

	let showBrowser = $state(false);
	let dropdownOpen = $state(false);

	// Get a display name for the selected sound
	const selectedName = $derived.by(() => {
		if (!value) return '';
		const sound = soundLibrary.getSoundByPath(value);
		if (sound) return sound.name;
		// Fall back to extracting filename from path
		const parts = value.split('/');
		return parts[parts.length - 1]?.replace(/\.[^.]+$/, '') || value;
	});

	onMount(async () => {
		if (soundLibrary.systemSounds.length === 0) {
			await soundLibrary.load();
		}
	});

	function handleSelect(sound: SystemSound) {
		onchange?.(sound.path);
		showBrowser = false;
		dropdownOpen = false;
	}

	async function handlePreview(e: Event) {
		e.stopPropagation();
		if (value) {
			await soundLibrary.previewSound(value);
		}
	}

	function handleQuickSelect(path: string) {
		onchange?.(path);
		dropdownOpen = false;
	}
</script>

<div class="relative">
	<!-- Main button -->
	<div class="flex gap-2">
		<button
			type="button"
			onclick={() => (dropdownOpen = !dropdownOpen)}
			class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600
				bg-white dark:bg-gray-800 hover:border-gray-400 dark:hover:border-gray-500 transition-colors text-left"
		>
			<Volume2 class="w-4 h-4 text-gray-400 flex-shrink-0" />
			{#if value}
				<span class="flex-1 truncate text-gray-900 dark:text-white">{selectedName}</span>
			{:else}
				<span class="flex-1 text-gray-500">{placeholder}</span>
			{/if}
			<ChevronDown class="w-4 h-4 text-gray-400" />
		</button>

		{#if value}
			<button
				type="button"
				onclick={handlePreview}
				class="px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600
					bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
				title="Preview sound"
			>
				{#if soundLibrary.isPlaying === value}
					<Pause class="w-4 h-4 text-orange-600" />
				{:else}
					<Play class="w-4 h-4 text-gray-600 dark:text-gray-400" />
				{/if}
			</button>
		{/if}
	</div>

	<!-- Dropdown -->
	{#if dropdownOpen}
		<!-- Backdrop to close dropdown when clicking outside -->
		<button
			type="button"
			class="fixed inset-0 z-40"
			onclick={() => (dropdownOpen = false)}
			aria-label="Close dropdown"
		></button>

		<div class="absolute z-50 mt-1 w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
			<!-- Quick picks from system sounds -->
			<div class="max-h-48 overflow-auto">
				{#each soundLibrary.systemSounds.slice(0, 8) as sound (sound.path)}
					<div
						role="button"
						tabindex="0"
						onclick={() => handleQuickSelect(sound.path)}
						onkeydown={(e) => e.key === 'Enter' && handleQuickSelect(sound.path)}
						class="w-full flex items-center gap-2 px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 text-left transition-colors cursor-pointer
							{value === sound.path ? 'bg-orange-50 dark:bg-orange-900/20' : ''}"
					>
						<button
							type="button"
							onclick={async (e) => {
								e.stopPropagation();
								await soundLibrary.previewSound(sound.path);
							}}
							class="w-6 h-6 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center hover:bg-orange-100 dark:hover:bg-orange-900/50"
						>
							{#if soundLibrary.isPlaying === sound.path}
								<Pause class="w-3 h-3 text-orange-600" />
							{:else}
								<Play class="w-3 h-3 text-gray-500" />
							{/if}
						</button>
						<span class="text-sm text-gray-900 dark:text-white">{sound.name}</span>
					</div>
				{/each}
			</div>

			<!-- Browse all button -->
			<button
				type="button"
				onclick={() => {
					dropdownOpen = false;
					showBrowser = true;
				}}
				class="w-full flex items-center gap-2 px-3 py-2 border-t border-gray-200 dark:border-gray-700
					text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/20 transition-colors"
			>
				<FolderOpen class="w-4 h-4" />
				<span class="text-sm font-medium">Browse all sounds...</span>
			</button>
		</div>
	{/if}
</div>

<!-- Full Browser Modal -->
{#if showBrowser}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden">
			<SoundBrowser
				onSelect={handleSelect}
				onClose={() => (showBrowser = false)}
				selectedPath={value}
			/>
		</div>
	</div>
{/if}
