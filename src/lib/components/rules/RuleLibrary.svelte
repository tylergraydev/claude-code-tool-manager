<script lang="ts">
	import type { Rule } from '$lib/types';
	import { ruleLibrary } from '$lib/stores';
	import RuleCard from './RuleCard.svelte';
	import { SearchBar } from '$lib/components/shared';
	import { BookOpen } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';

	type Props = {
		onEdit?: (rule: Rule) => void;
		onDelete?: (rule: Rule) => void;
	};

	let { onEdit, onDelete }: Props = $props();

	async function handleFavoriteToggle(rule: Rule, favorite: boolean) {
		try {
			await invoke('toggle_rule_favorite', { id: rule.id, favorite });
			ruleLibrary.updateRule({ ...rule, isFavorite: favorite });
		} catch (error) {
			console.error('Failed to toggle favorite:', error);
		}
	}
</script>

<div class="space-y-4">
	<!-- Filters -->
	<div class="flex items-center gap-4">
		<div class="flex-1 max-w-sm">
			<SearchBar
				bind:value={ruleLibrary.searchQuery}
				placeholder="Search rules..."
			/>
		</div>

		<div class="text-sm text-gray-500 dark:text-gray-400">
			{ruleLibrary.rules.length} rule{ruleLibrary.rules.length !== 1 ? 's' : ''}
		</div>
	</div>

	<!-- Rule Grid -->
	{#if ruleLibrary.isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if ruleLibrary.filteredRules.length === 0}
		<div class="text-center py-12">
			<BookOpen class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
			{#if ruleLibrary.searchQuery}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No matching rules</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Try adjusting your search
				</p>
			{:else}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No rules in library</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Add your first rule to provide conditional instructions to Claude
				</p>
			{/if}
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
			{#each ruleLibrary.filteredRules as rule (rule.id)}
				<RuleCard
					{rule}
					{onEdit}
					{onDelete}
					onFavoriteToggle={handleFavoriteToggle}
				/>
			{/each}
		</div>
	{/if}
</div>
