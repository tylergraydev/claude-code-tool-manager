<script lang="ts">
	import { profileLibrary } from '$lib/stores';
	import type { Profile } from '$lib/types';
	import { Search } from 'lucide-svelte';
	import ProfileCard from './ProfileCard.svelte';

	type Props = {
		onActivate?: (profile: Profile) => void;
		onDeactivate?: () => void;
		onCapture?: (profile: Profile) => void;
		onEdit?: (profile: Profile) => void;
		onDelete?: (profile: Profile) => void;
	};

	let { onActivate, onDeactivate, onCapture, onEdit, onDelete }: Props = $props();
</script>

<!-- Search -->
{#if profileLibrary.profiles.length > 0}
	<div class="mb-4">
		<div class="relative">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
			<input
				type="text"
				placeholder="Search profiles..."
				value={profileLibrary.searchQuery}
				oninput={(e) => profileLibrary.setSearch(e.currentTarget.value)}
				class="w-full pl-10 pr-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:border-transparent text-sm"
			/>
		</div>
	</div>
{/if}

{#if profileLibrary.isLoading}
	<div class="flex items-center justify-center py-12">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
	</div>
{:else if profileLibrary.profiles.length === 0}
	<div class="text-center py-12">
		<p class="text-gray-500 dark:text-gray-400">No profiles yet. Create one to get started.</p>
	</div>
{:else if profileLibrary.filteredProfiles.length === 0}
	<div class="text-center py-12">
		<p class="text-gray-500 dark:text-gray-400">No profiles match your search.</p>
	</div>
{:else}
	<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
		{#each profileLibrary.filteredProfiles as profile (profile.id)}
			<ProfileCard
				{profile}
				{onActivate}
				{onDeactivate}
				{onCapture}
				{onEdit}
				{onDelete}
			/>
		{/each}
	</div>
{/if}
