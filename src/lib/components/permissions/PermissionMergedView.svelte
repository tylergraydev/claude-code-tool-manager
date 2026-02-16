<script lang="ts">
	import type { PermissionCategory } from '$lib/types';
	import { ShieldAlert, ShieldQuestion, ShieldCheck, Layers } from 'lucide-svelte';

	type MergedRule = {
		rule: string;
		category: PermissionCategory;
		scope: string;
	};

	type Props = {
		rules: MergedRule[];
		onclose: () => void;
	};

	let { rules, onclose }: Props = $props();

	const categoryIcons: Record<PermissionCategory, typeof ShieldAlert> = {
		deny: ShieldAlert,
		ask: ShieldQuestion,
		allow: ShieldCheck
	};

	const categoryColors: Record<PermissionCategory, string> = {
		deny: 'text-red-600 dark:text-red-400',
		ask: 'text-amber-600 dark:text-amber-400',
		allow: 'text-green-600 dark:text-green-400'
	};

	const categoryBadgeColors: Record<PermissionCategory, string> = {
		deny: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300',
		ask: 'bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300',
		allow: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300'
	};

	const scopeBadgeColors: Record<string, string> = {
		user: 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300',
		project: 'bg-purple-100 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300',
		local: 'bg-gray-100 dark:bg-gray-600 text-gray-700 dark:text-gray-300'
	};
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onkeydown={(e) => e.key === 'Escape' && onclose()}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div
			class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700"
		>
			<div class="flex items-center gap-2">
				<Layers class="w-5 h-5 text-primary-500" />
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
					Merged Permissions View
				</h3>
			</div>
			<button
				onclick={onclose}
				class="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
			>
				&times;
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-6">
			<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
				Combined view of all permission rules across all scopes. Rules are evaluated in order:
				deny first, then ask, then allow.
			</p>

			{#if rules.length === 0}
				<div class="text-center py-12 text-gray-400 dark:text-gray-500">
					<p>No permission rules configured</p>
				</div>
			{:else}
				<div class="space-y-1.5">
					{#each rules as { rule, category, scope }}
						<div
							class="flex items-center gap-3 px-4 py-2.5 rounded-lg bg-gray-50 dark:bg-gray-700/30"
						>
							<svelte:component
								this={categoryIcons[category]}
								class="w-4 h-4 {categoryColors[category]}"
							/>
							<code class="flex-1 text-sm font-mono text-gray-800 dark:text-gray-200">{rule}</code>
							<span class="text-xs px-2 py-0.5 rounded-full {categoryBadgeColors[category]}">
								{category}
							</span>
							<span
								class="text-xs px-2 py-0.5 rounded-full {scopeBadgeColors[scope] ?? 'bg-gray-100 text-gray-600'}"
							>
								{scope}
							</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div
			class="px-6 py-3 border-t border-gray-200 dark:border-gray-700 flex justify-end"
		>
			<button onclick={onclose} class="btn btn-secondary">Close</button>
		</div>
	</div>
</div>
