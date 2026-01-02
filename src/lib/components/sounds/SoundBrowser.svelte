<script lang="ts">
	import { onMount } from 'svelte';
	import { soundLibrary, notifications } from '$lib/stores';
	import type { SystemSound, CustomSound } from '$lib/types';
	import { Play, Pause, Volume2, Upload, Trash2, FolderOpen, X } from 'lucide-svelte';

	type Props = {
		onSelect?: (sound: SystemSound) => void;
		onClose?: () => void;
		selectedPath?: string;
		showCustomUpload?: boolean;
	};

	let { onSelect, onClose, selectedPath = '', showCustomUpload = true }: Props = $props();

	let activeTab = $state<'system' | 'custom'>('system');
	let dragOver = $state(false);
	let fileInput: HTMLInputElement;

	onMount(async () => {
		if (soundLibrary.systemSounds.length === 0) {
			await soundLibrary.load();
		}
	});

	function handleSelect(sound: SystemSound) {
		onSelect?.(sound);
	}

	async function handlePlay(sound: SystemSound, e: Event) {
		e.stopPropagation();
		await soundLibrary.previewSound(sound.path);
	}

	async function handleFileUpload(files: FileList | null) {
		if (!files) return;

		for (const file of files) {
			if (!file.name.match(/\.(wav|mp3|aiff|ogg|m4a)$/i)) {
				notifications.error(`Invalid file type: ${file.name}`);
				continue;
			}

			try {
				const data = new Uint8Array(await file.arrayBuffer());
				await soundLibrary.uploadSound(file.name, data);
				notifications.success(`Uploaded: ${file.name}`);
			} catch (e) {
				notifications.error(`Failed to upload ${file.name}`);
			}
		}
	}

	async function handleDelete(sound: CustomSound, e: Event) {
		e.stopPropagation();
		try {
			await soundLibrary.deleteSound(sound.name);
			notifications.success(`Deleted: ${sound.name}`);
		} catch (e) {
			notifications.error('Failed to delete sound');
		}
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
		handleFileUpload(e.dataTransfer?.files ?? null);
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dragOver = true;
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
	}
</script>

<div class="flex flex-col h-full max-h-[70vh]">
	<!-- Header -->
	<div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
		<div class="flex items-center gap-2">
			<Volume2 class="w-5 h-5 text-gray-500" />
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Sound Browser</h3>
		</div>
		{#if onClose}
			<button onclick={onClose} class="p-1 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
				<X class="w-5 h-5" />
			</button>
		{/if}
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-gray-200 dark:border-gray-700">
		<button
			onclick={() => (activeTab = 'system')}
			class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'system'
				? 'text-orange-600 border-b-2 border-orange-500'
				: 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
		>
			System Sounds ({soundLibrary.systemSounds.length})
		</button>
		<button
			onclick={() => (activeTab = 'custom')}
			class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'custom'
				? 'text-orange-600 border-b-2 border-orange-500'
				: 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
		>
			Custom Sounds ({soundLibrary.customSounds.length})
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-auto p-4">
		{#if soundLibrary.isLoading}
			<div class="flex items-center justify-center py-12">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
			</div>
		{:else if activeTab === 'system'}
			<!-- System Sounds Grid -->
			{#if soundLibrary.systemSounds.length === 0}
				<div class="text-center py-12 text-gray-500">
					<Volume2 class="w-12 h-12 mx-auto mb-4 opacity-50" />
					<p>No system sounds found</p>
				</div>
			{:else}
				<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
					{#each soundLibrary.systemSounds as sound (sound.path)}
						<div
							role="button"
							tabindex="0"
							onclick={() => handleSelect(sound)}
							onkeydown={(e) => e.key === 'Enter' && handleSelect(sound)}
							class="flex items-center gap-2 p-3 rounded-lg border-2 transition-all text-left cursor-pointer
								{selectedPath === sound.path
								? 'border-orange-500 bg-orange-50 dark:bg-orange-900/20'
								: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
						>
							<button
								onclick={(e) => handlePlay(sound, e)}
								class="flex-shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center
									hover:bg-orange-100 dark:hover:bg-orange-900/50 transition-colors"
							>
								{#if soundLibrary.isPlaying === sound.path}
									<Pause class="w-4 h-4 text-orange-600" />
								{:else}
									<Play class="w-4 h-4 text-gray-600 dark:text-gray-400" />
								{/if}
							</button>
							<span class="text-sm font-medium text-gray-900 dark:text-white truncate">
								{sound.name}
							</span>
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			<!-- Custom Sounds -->
			{#if showCustomUpload}
				<!-- Upload Zone -->
				<div
					ondrop={handleDrop}
					ondragover={handleDragOver}
					ondragleave={handleDragLeave}
					class="mb-4 p-6 border-2 border-dashed rounded-xl transition-colors
						{dragOver
						? 'border-orange-500 bg-orange-50 dark:bg-orange-900/20'
						: 'border-gray-300 dark:border-gray-600'}"
				>
					<div class="flex flex-col items-center text-center">
						<Upload class="w-8 h-8 text-gray-400 mb-2" />
						<p class="text-sm text-gray-600 dark:text-gray-400 mb-2">
							Drag and drop sound files here, or
						</p>
						<button
							onclick={() => fileInput?.click()}
							class="btn btn-secondary text-sm"
						>
							Browse Files
						</button>
						<input
							bind:this={fileInput}
							type="file"
							accept=".wav,.mp3,.aiff,.ogg,.m4a"
							multiple
							class="hidden"
							onchange={(e) => handleFileUpload((e.target as HTMLInputElement).files)}
						/>
						<p class="text-xs text-gray-500 mt-2">Supports: WAV, MP3, AIFF, OGG, M4A</p>
					</div>
				</div>
			{/if}

			<!-- Custom Sounds List -->
			{#if soundLibrary.customSounds.length === 0}
				<div class="text-center py-8 text-gray-500">
					<FolderOpen class="w-12 h-12 mx-auto mb-4 opacity-50" />
					<p>No custom sounds yet</p>
					<p class="text-sm mt-1">Upload sounds to use them in hooks</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each soundLibrary.customSounds as sound (sound.path)}
						<div
							class="flex items-center gap-3 p-3 rounded-lg border-2 transition-all
								{selectedPath === sound.path
								? 'border-orange-500 bg-orange-50 dark:bg-orange-900/20'
								: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
						>
							<button
								onclick={() => handlePlay({ name: sound.name, path: sound.path, category: 'custom' }, new Event('click'))}
								class="flex-shrink-0 w-10 h-10 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center
									hover:bg-orange-100 dark:hover:bg-orange-900/50 transition-colors"
							>
								{#if soundLibrary.isPlaying === sound.path}
									<Pause class="w-5 h-5 text-orange-600" />
								{:else}
									<Play class="w-5 h-5 text-gray-600 dark:text-gray-400" />
								{/if}
							</button>

							<button
								onclick={() => handleSelect({ name: sound.name, path: sound.path, category: 'custom' })}
								class="flex-1 text-left"
							>
								<p class="font-medium text-gray-900 dark:text-white">{sound.name}</p>
								<p class="text-xs text-gray-500">
									{(sound.size / 1024).toFixed(1)} KB
								</p>
							</button>

							<button
								onclick={(e) => handleDelete(sound, e)}
								class="p-2 text-gray-400 hover:text-red-500 transition-colors"
								title="Delete sound"
							>
								<Trash2 class="w-4 h-4" />
							</button>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>

	<!-- Footer with sounds directory info -->
	<div class="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
		<p class="text-xs text-gray-500 font-mono truncate">
			{soundLibrary.soundsDirectory || '~/.claude/sounds/'}
		</p>
	</div>
</div>
