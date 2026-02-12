<script lang="ts">
	import type { Profile } from '$lib/types';
	import { Layers, Play, Camera, Edit, Trash2, MoreVertical, X } from 'lucide-svelte';

	type Props = {
		profile: Profile;
		onActivate?: (profile: Profile) => void;
		onDeactivate?: () => void;
		onCapture?: (profile: Profile) => void;
		onEdit?: (profile: Profile) => void;
		onDelete?: (profile: Profile) => void;
	};

	let { profile, onActivate, onDeactivate, onCapture, onEdit, onDelete }: Props = $props();

	let showMenu = $state(false);
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border transition-all hover:shadow-md
		{profile.isActive
			? 'border-green-300 dark:border-green-600 ring-1 ring-green-200 dark:ring-green-700'
			: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
>
	<div class="p-4">
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-3 min-w-0">
				<div
					class="w-10 h-10 rounded-lg flex items-center justify-center text-lg shrink-0
						{profile.isActive
							? 'bg-green-100 dark:bg-green-900/50'
							: 'bg-gray-100 dark:bg-gray-700'}"
				>
					{#if profile.icon}
						{profile.icon}
					{:else}
						<Layers class="w-5 h-5 {profile.isActive ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}" />
					{/if}
				</div>
				<div class="min-w-0">
					<div class="flex items-center gap-2">
						<h3 class="font-medium text-gray-900 dark:text-white truncate">{profile.name}</h3>
						{#if profile.isActive}
							<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400">
								Active
							</span>
						{/if}
					</div>
					{#if profile.description}
						<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">
							{profile.description}
						</p>
					{/if}
				</div>
			</div>

			<!-- Actions menu -->
			<div class="relative shrink-0 ml-2">
				<button
					onclick={(e) => { e.stopPropagation(); showMenu = !showMenu; }}
					class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
				>
					<MoreVertical class="w-4 h-4" />
				</button>

				{#if showMenu}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="fixed inset-0 z-40" onclick={() => (showMenu = false)}></div>
					<div class="absolute right-0 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50">
						<button
							onclick={() => { showMenu = false; onEdit?.(profile); }}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
						>
							<Edit class="w-4 h-4" />
							Edit
						</button>
						<button
							onclick={() => { showMenu = false; onCapture?.(profile); }}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
						>
							<Camera class="w-4 h-4" />
							Capture Current
						</button>
						<hr class="my-1 border-gray-200 dark:border-gray-700" />
						<button
							onclick={() => { showMenu = false; onDelete?.(profile); }}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
						>
							<Trash2 class="w-4 h-4" />
							Delete
						</button>
					</div>
				{/if}
			</div>
		</div>

		<!-- Action buttons -->
		<div class="mt-4 flex gap-2">
			{#if profile.isActive}
				<button
					onclick={() => onDeactivate?.()}
					class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
				>
					<X class="w-3.5 h-3.5" />
					Deactivate
				</button>
			{:else}
				<button
					onclick={() => onActivate?.(profile)}
					class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition-colors"
				>
					<Play class="w-3.5 h-3.5" />
					Activate
				</button>
			{/if}
		</div>
	</div>
</div>
