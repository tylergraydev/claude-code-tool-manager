<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import {
		PermissionRuleList,
		PermissionRuleForm,
		PermissionScopeSelector,
		PermissionTemplatePanel,
		PermissionMergedView,
		DefaultModeSelector,
		AdditionalDirectoriesEditor
	} from '$lib/components/permissions';
	import { SearchBar } from '$lib/components/shared';
	import { permissionLibrary, projectsStore, notifications } from '$lib/stores';
	import type { PermissionCategory, PermissionTemplate } from '$lib/types';
	import { Sparkles, Layers, RefreshCw, FolderOpen } from 'lucide-svelte';

	let showRuleForm = $state(false);
	let ruleFormCategory = $state<PermissionCategory>('allow');
	let showTemplatePanel = $state(false);
	let showMergedView = $state(false);

	onMount(async () => {
		await projectsStore.loadProjects();
		await permissionLibrary.load();
		await permissionLibrary.seedTemplates();
		await permissionLibrary.loadTemplates();
	});

	function openAddRule(category: PermissionCategory) {
		ruleFormCategory = category;
		showRuleForm = true;
	}

	async function handleAddRule(rule: string) {
		try {
			await permissionLibrary.addRule(ruleFormCategory, rule);
			showRuleForm = false;
			notifications.success(`Rule added to ${ruleFormCategory}`);
		} catch (err) {
			notifications.error('Failed to add rule');
		}
	}

	async function handleRemoveRule(category: PermissionCategory, index: number) {
		try {
			await permissionLibrary.removeRule(category, index);
			notifications.success('Rule removed');
		} catch (err) {
			notifications.error('Failed to remove rule');
		}
	}

	async function handleReorder(category: PermissionCategory, rules: string[]) {
		try {
			await permissionLibrary.reorderRules(category, rules);
		} catch (err) {
			notifications.error('Failed to reorder rules');
		}
	}

	async function handleApplyTemplate(template: PermissionTemplate) {
		try {
			await permissionLibrary.applyTemplate(template);
			showTemplatePanel = false;
			notifications.success(`Applied template: ${template.name}`);
		} catch (err) {
			notifications.error('Failed to apply template');
		}
	}

	async function handleDefaultModeChange(mode: string | null) {
		try {
			await permissionLibrary.setDefaultMode(mode);
			notifications.success('Default mode updated');
		} catch (err) {
			notifications.error('Failed to update default mode');
		}
	}

	async function handleDirectoriesChange(dirs: string[]) {
		try {
			await permissionLibrary.setAdditionalDirectories(dirs);
			notifications.success('Additional directories updated');
		} catch (err) {
			notifications.error('Failed to update directories');
		}
	}

	function handleProjectChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		const value = target.value;
		permissionLibrary.setProjectPath(value || null);
		permissionLibrary.load();
	}

	async function handleRefresh() {
		await permissionLibrary.load();
		notifications.success('Permissions refreshed');
	}
</script>

<Header
	title="Permissions"
	subtitle="Manage Claude Code permission rules — control what tools Claude can use"
/>

<div class="flex-1 overflow-auto p-6">
	<!-- Top bar: Project selector + Scope tabs + Actions -->
	<div class="flex flex-wrap items-center gap-4 mb-6">
		<!-- Project selector -->
		<div class="flex items-center gap-2">
			<FolderOpen class="w-4 h-4 text-gray-500 dark:text-gray-400" />
			<select
				value={permissionLibrary.projectPath ?? ''}
				onchange={handleProjectChange}
				class="input text-sm"
			>
				<option value="">No project</option>
				{#each projectsStore.projects as project}
					<option value={project.path}>{project.name}</option>
				{/each}
			</select>
		</div>

		<!-- Scope selector -->
		<div class="flex-1 min-w-[300px]">
			<PermissionScopeSelector
				selectedScope={permissionLibrary.selectedScope}
				permissions={permissionLibrary.permissions}
				hasProject={!!permissionLibrary.projectPath}
				onselect={(scope) => permissionLibrary.setScope(scope)}
			/>
		</div>

		<!-- Actions -->
		<div class="flex items-center gap-2">
			<button
				onclick={() => (showTemplatePanel = true)}
				class="btn btn-secondary"
			>
				<Sparkles class="w-4 h-4 mr-2" />
				Templates
			</button>
			<button
				onclick={() => (showMergedView = true)}
				class="btn btn-secondary"
			>
				<Layers class="w-4 h-4 mr-2" />
				Merged View
			</button>
			<button
				onclick={handleRefresh}
				class="btn btn-ghost"
				title="Refresh from settings files"
			>
				<RefreshCw class="w-4 h-4" />
			</button>
		</div>
	</div>

	{#if permissionLibrary.isLoading}
		<div class="flex items-center justify-center py-20">
			<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"></div>
		</div>
	{:else if permissionLibrary.error}
		<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400">
			{permissionLibrary.error}
		</div>
	{:else if permissionLibrary.currentScopePermissions}
		<!-- Settings row -->
		<div class="flex flex-wrap items-start gap-6 mb-6 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
			<DefaultModeSelector
				value={permissionLibrary.currentScopePermissions.defaultMode}
				onchange={handleDefaultModeChange}
			/>
			<div class="border-l border-gray-200 dark:border-gray-700 pl-6 flex-1 min-w-[250px]">
				<AdditionalDirectoriesEditor
					directories={permissionLibrary.currentScopePermissions.additionalDirectories}
					onchange={handleDirectoriesChange}
				/>
			</div>
		</div>

		<!-- Search -->
		<div class="mb-4 max-w-md">
			<SearchBar
				value={permissionLibrary.searchQuery}
				placeholder="Filter rules..."
				onchange={(v) => permissionLibrary.setSearch(v)}
			/>
		</div>

		<!-- Rule lists -->
		<div class="space-y-4">
			<PermissionRuleList
				category="deny"
				rules={permissionLibrary.filteredRules.deny}
				onremove={(index) => handleRemoveRule('deny', index)}
				onadd={() => openAddRule('deny')}
				onreorder={(rules) => handleReorder('deny', rules)}
			/>

			<PermissionRuleList
				category="ask"
				rules={permissionLibrary.filteredRules.ask}
				onremove={(index) => handleRemoveRule('ask', index)}
				onadd={() => openAddRule('ask')}
				onreorder={(rules) => handleReorder('ask', rules)}
			/>

			<PermissionRuleList
				category="allow"
				rules={permissionLibrary.filteredRules.allow}
				onremove={(index) => handleRemoveRule('allow', index)}
				onadd={() => openAddRule('allow')}
				onreorder={(rules) => handleReorder('allow', rules)}
			/>
		</div>
	{:else}
		<div class="text-center py-20 text-gray-400 dark:text-gray-500">
			<p>Select a scope to view permissions</p>
		</div>
	{/if}
</div>

<!-- Modals -->
{#if showRuleForm}
	<PermissionRuleForm
		category={ruleFormCategory}
		onsubmit={handleAddRule}
		onclose={() => (showRuleForm = false)}
	/>
{/if}

{#if showTemplatePanel}
	<PermissionTemplatePanel
		templates={permissionLibrary.templates}
		onApply={handleApplyTemplate}
		onclose={() => (showTemplatePanel = false)}
	/>
{/if}

{#if showMergedView}
	<PermissionMergedView
		rules={permissionLibrary.mergedView}
		onclose={() => (showMergedView = false)}
	/>
{/if}
