<script lang="ts">
	import type { Project, ProjectSkill, ProjectSubAgent } from '$lib/types';
	import { projectsStore, notifications, skillLibrary, subagentLibrary } from '$lib/stores';
	import { FolderOpen, Trash2, RefreshCw, ExternalLink, Plug, Sparkles, Bot } from 'lucide-svelte';
	import { open } from '@tauri-apps/plugin-shell';
	import { ActionMenu, ActionMenuItem, FavoriteButton, Badge } from '$lib/components/shared';

	type Props = {
		project: Project;
		preloadedSkills?: ProjectSkill[];
		preloadedAgents?: ProjectSubAgent[];
		onRemove?: (project: Project) => void;
		onClick?: () => void;
		onFavoriteToggle?: (project: Project, favorite: boolean) => void;
	};

	let { project, preloadedSkills, preloadedAgents, onRemove, onClick, onFavoriteToggle }: Props = $props();

	let actionMenu: ActionMenu;

	// Skills and agents for this project — use preloaded data or fetch on demand
	let projectSkills = $state<ProjectSkill[]>([]);
	let projectAgents = $state<ProjectSubAgent[]>([]);

	$effect(() => {
		if (preloadedSkills !== undefined) {
			projectSkills = preloadedSkills;
		} else {
			loadProjectSkills();
		}
	});

	$effect(() => {
		if (preloadedAgents !== undefined) {
			projectAgents = preloadedAgents;
		} else {
			loadProjectAgents();
		}
	});

	async function loadProjectSkills() {
		try {
			projectSkills = await skillLibrary.getProjectSkills(project.id);
		} catch (err) {
			console.error('Failed to load project skills:', err);
		}
	}

	async function loadProjectAgents() {
		try {
			projectAgents = await subagentLibrary.getProjectSubAgents(project.id);
		} catch (err) {
			console.error('Failed to load project agents:', err);
		}
	}

	// Count enabled vs total MCPs
	let enabledMcpCount = $derived(project.assignedMcps.filter((a) => a.isEnabled).length);
	let totalMcpCount = $derived(project.assignedMcps.length);

	// Count enabled vs total Skills
	let enabledSkillCount = $derived(projectSkills.filter((s) => s.isEnabled).length);
	let totalSkillCount = $derived(projectSkills.length);

	// Count enabled vs total Agents
	let enabledAgentCount = $derived(projectAgents.filter((a) => a.isEnabled).length);
	let totalAgentCount = $derived(projectAgents.length);
</script>

<div
	class="card transition-shadow duration-200 hover:shadow-md cursor-pointer hover:border-primary-300 dark:hover:border-primary-700"
	onclick={onClick}
	role="button"
	tabindex="0"
	onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onClick?.())}
>
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
			<FolderOpen class="w-5 h-5 text-amber-600 dark:text-amber-400" aria-hidden="true" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{project.name}
				</h3>
				{#if project.editorType === 'opencode'}
					<Badge variant="auto">
						<span class="w-3 h-3 rounded-sm bg-emerald-500 text-white flex items-center justify-center text-[8px] font-bold" aria-hidden="true">O</span>
						OpenCode
					</Badge>
				{:else}
					<Badge variant="info">
						<span class="w-3 h-3 rounded-sm bg-primary-500 text-white flex items-center justify-center text-[8px] font-bold" aria-hidden="true">C</span>
						Claude
					</Badge>
				{/if}
				{#if project.hasMcpFile}
					<Badge variant="success">.mcp.json</Badge>
				{/if}
			</div>
			<p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5 font-mono">
				{project.path}
			</p>
		</div>

		<div class="flex items-center gap-1">
			{#if onFavoriteToggle}
				<FavoriteButton
					isFavorite={project.isFavorite}
					name={project.name}
					onclick={() => onFavoriteToggle(project, !project.isFavorite)}
				/>
			{/if}
			<ActionMenu bind:this={actionMenu} label="Actions for {project.name}">
				<ActionMenuItem icon={RefreshCw} label="Sync Config" onclick={async () => {
					try {
						await projectsStore.syncProjectConfig(project.id);
						notifications.success('Config synced');
					} catch {
						notifications.error('Failed to sync config');
					}
					actionMenu.close();
				}} />
				<ActionMenuItem icon={ExternalLink} label="Open Folder" onclick={async () => {
					try {
						await open(project.path);
					} catch {
						notifications.error('Failed to open folder');
					}
					actionMenu.close();
				}} />
				{#if onRemove}
					<ActionMenuItem icon={Trash2} label="Remove" variant="danger" onclick={() => {
						onRemove(project);
						actionMenu.close();
					}} />
				{/if}
			</ActionMenu>
		</div>
	</div>

	<!-- Counts -->
	<div class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1">
		<div class="flex items-center gap-1.5 text-sm">
			<Plug class="w-4 h-4 text-purple-400" aria-hidden="true" />
			{#if totalMcpCount > 0}
				<span class="text-gray-600 dark:text-gray-300" aria-label="{enabledMcpCount} of {totalMcpCount} MCPs enabled">
					{enabledMcpCount}/{totalMcpCount}
				</span>
			{:else}
				<span class="text-gray-400 dark:text-gray-500">0</span>
			{/if}
		</div>

		<div class="flex items-center gap-1.5 text-sm">
			<Sparkles class="w-4 h-4 text-yellow-400" aria-hidden="true" />
			{#if totalSkillCount > 0}
				<span class="text-gray-600 dark:text-gray-300" aria-label="{enabledSkillCount} of {totalSkillCount} skills enabled">
					{enabledSkillCount}/{totalSkillCount}
				</span>
			{:else}
				<span class="text-gray-400 dark:text-gray-500">0</span>
			{/if}
		</div>

		<div class="flex items-center gap-1.5 text-sm">
			<Bot class="w-4 h-4 text-cyan-400" aria-hidden="true" />
			{#if totalAgentCount > 0}
				<span class="text-gray-600 dark:text-gray-300" aria-label="{enabledAgentCount} of {totalAgentCount} agents enabled">
					{enabledAgentCount}/{totalAgentCount}
				</span>
			{:else}
				<span class="text-gray-400 dark:text-gray-500">0</span>
			{/if}
		</div>
	</div>
</div>
