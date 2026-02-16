<script lang="ts">
	import type { PermissionTemplate, PermissionCategory } from '$lib/types';
	import {
		X,
		ShieldAlert,
		ShieldQuestion,
		ShieldCheck,
		Sparkles
	} from 'lucide-svelte';

	type Props = {
		templates: PermissionTemplate[];
		onApply: (template: PermissionTemplate) => void;
		onclose: () => void;
	};

	let { templates, onApply, onclose }: Props = $props();

	const grouped = $derived.by(() => {
		const groups: Record<PermissionCategory, PermissionTemplate[]> = {
			deny: [],
			ask: [],
			allow: []
		};
		for (const t of templates) {
			if (groups[t.category]) {
				groups[t.category].push(t);
			}
		}
		return groups;
	});

	const categoryConfig: Record<
		PermissionCategory,
		{ label: string; icon: typeof ShieldAlert; color: string; badgeColor: string }
	> = {
		deny: {
			label: 'Deny',
			icon: ShieldAlert,
			color: 'text-red-600 dark:text-red-400',
			badgeColor: 'bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300'
		},
		ask: {
			label: 'Ask',
			icon: ShieldQuestion,
			color: 'text-amber-600 dark:text-amber-400',
			badgeColor: 'bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300'
		},
		allow: {
			label: 'Allow',
			icon: ShieldCheck,
			color: 'text-green-600 dark:text-green-400',
			badgeColor: 'bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300'
		}
	};

	const categories: PermissionCategory[] = ['deny', 'ask', 'allow'];
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex justify-end bg-black/30"
	onkeydown={(e) => e.key === 'Escape' && onclose()}
>
	<div
		class="w-full max-w-md bg-white dark:bg-gray-800 shadow-xl overflow-y-auto"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div
			class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700 sticky top-0 bg-white dark:bg-gray-800 z-10"
		>
			<div class="flex items-center gap-2">
				<Sparkles class="w-5 h-5 text-primary-500" />
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Rule Templates</h3>
			</div>
			<button
				onclick={onclose}
				class="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Templates by category -->
		<div class="p-4 space-y-6">
			{#each categories as cat}
				{@const config = categoryConfig[cat]}
				{@const catTemplates = grouped[cat]}
				{#if catTemplates.length > 0}
					<div>
						<div class="flex items-center gap-2 mb-3">
							<svelte:component this={config.icon} class="w-4 h-4 {config.color}" />
							<h4 class="text-sm font-semibold {config.color}">{config.label}</h4>
						</div>
						<div class="space-y-2">
							{#each catTemplates as template}
								<button
									onclick={() => onApply(template)}
									class="w-full text-left px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-600 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
								>
									<div class="flex items-center justify-between mb-1">
										<span class="text-sm font-medium text-gray-900 dark:text-white">
											{template.name}
										</span>
										<span class="text-xs px-2 py-0.5 rounded-full {config.badgeColor}">
											{cat}
										</span>
									</div>
									<code class="text-xs font-mono text-gray-600 dark:text-gray-400">
										{template.rule}
									</code>
									{#if template.description}
										<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
											{template.description}
										</p>
									{/if}
								</button>
							{/each}
						</div>
					</div>
				{/if}
			{/each}

			{#if templates.length === 0}
				<div class="text-center py-8 text-gray-400 dark:text-gray-500">
					<p class="text-sm">No templates available</p>
				</div>
			{/if}
		</div>
	</div>
</div>
