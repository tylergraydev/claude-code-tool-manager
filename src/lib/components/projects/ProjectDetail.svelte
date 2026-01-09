<script lang="ts">
	import type { Project, Mcp, Skill, SubAgent, Command, ProjectSkill, ProjectSubAgent, ProjectCommand } from '$lib/types';
	import { mcpLibrary, projectsStore, notifications, skillLibrary, subagentLibrary, commandLibrary } from '$lib/stores';
	import { X, Plus, Minus, FolderOpen, Plug, Globe, Server, Sparkles, Bot, ChevronDown, Terminal, Search } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';

	type Props = {
		project: Project;
		onClose: () => void;
	};

	let { project: initialProject, onClose }: Props = $props();

	let showEditorDropdown = $state(false);
	let updatingEditor = $state(false);

	// Tab state
	type Tab = 'mcps' | 'skills' | 'agents' | 'commands';
	let activeTab = $state<Tab>('mcps');

	// Search state
	let searchQuery = $state('');

	const typeIcons = {
		stdio: Plug,
		sse: Globe,
		http: Server
	};

	const typeColors = {
		stdio: 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400',
		sse: 'bg-green-100 text-green-600 dark:bg-green-900/50 dark:text-green-400',
		http: 'bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400'
	};

	// Get current project from store (updates after loadProjects)
	let project = $derived(
		projectsStore.getProjectById(initialProject.id) ?? initialProject
	);

	// Get assigned MCP IDs for this project
	let assignedMcpIds = $derived(project.assignedMcps.map((a) => a.mcpId));

	// Available MCPs (in library but not assigned to this project)
	let availableMcps = $derived(
		mcpLibrary.mcps.filter((mcp) => !assignedMcpIds.includes(mcp.id))
	);

	// Filtered available MCPs based on search
	let filteredAvailableMcps = $derived(
		searchQuery.trim()
			? availableMcps.filter((mcp) => {
					const query = searchQuery.toLowerCase();
					return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
				})
			: availableMcps
	);

	// Skills state
	let projectSkills = $state<ProjectSkill[]>([]);
	let assignedSkillIds = $derived(projectSkills.map((ps) => ps.skillId));
	let availableSkills = $derived(
		skillLibrary.skills.filter((skill) => !assignedSkillIds.includes(skill.id))
	);

	// Filtered available Skills based on search
	let filteredAvailableSkills = $derived(
		searchQuery.trim()
			? availableSkills.filter((skill) => {
					const query = searchQuery.toLowerCase();
					return skill.name.toLowerCase().includes(query) || skill.description?.toLowerCase().includes(query);
				})
			: availableSkills
	);

	// SubAgents state
	let projectSubAgents = $state<ProjectSubAgent[]>([]);
	let assignedSubAgentIds = $derived(projectSubAgents.map((pa) => pa.subagentId));
	let availableSubAgents = $derived(
		subagentLibrary.subagents.filter((agent) => !assignedSubAgentIds.includes(agent.id))
	);

	// Filtered available SubAgents based on search
	let filteredAvailableSubAgents = $derived(
		searchQuery.trim()
			? availableSubAgents.filter((agent) => {
					const query = searchQuery.toLowerCase();
					return agent.name.toLowerCase().includes(query) || agent.description?.toLowerCase().includes(query);
				})
			: availableSubAgents
	);

	// Commands state
	let projectCommands = $state<ProjectCommand[]>([]);
	let assignedCommandIds = $derived(projectCommands.map((pc) => pc.commandId));
	let availableCommands = $derived(
		commandLibrary.commands.filter((cmd) => !assignedCommandIds.includes(cmd.id))
	);

	// Filtered available Commands based on search
	let filteredAvailableCommands = $derived(
		searchQuery.trim()
			? availableCommands.filter((cmd) => {
					const query = searchQuery.toLowerCase();
					return cmd.name.toLowerCase().includes(query) || cmd.description?.toLowerCase().includes(query);
				})
			: availableCommands
	);

	// Load project skills, subagents, and commands
	$effect(() => {
		loadProjectData();
	});

	async function loadProjectData() {
		try {
			projectSkills = await skillLibrary.getProjectSkills(project.id);
			projectSubAgents = await subagentLibrary.getProjectSubAgents(project.id);
			projectCommands = await commandLibrary.getProjectCommands(project.id);
		} catch (err) {
			console.error('Failed to load project data:', err);
		}
	}

	// MCP handlers
	async function handleAddMcp(mcp: Mcp) {
		try {
			await projectsStore.assignMcpToProject(project.id, mcp.id);
			await projectsStore.syncProjectConfig(project.id);
			notifications.success(`Added ${mcp.name} to ${project.name}`);
		} catch (err) {
			notifications.error('Failed to add MCP');
			console.error(err);
		}
	}

	async function handleRemoveMcp(mcpId: number) {
		try {
			const mcp = mcpLibrary.getMcpById(mcpId);
			await projectsStore.removeMcpFromProject(project.id, mcpId);
			await projectsStore.syncProjectConfig(project.id);
			notifications.success(`Removed ${mcp?.name || 'MCP'} from ${project.name}`);
		} catch (err) {
			notifications.error('Failed to remove MCP');
			console.error(err);
		}
	}

	async function handleToggleMcp(assignmentId: number, enabled: boolean) {
		try {
			await projectsStore.toggleProjectMcp(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
		} catch (err) {
			notifications.error('Failed to toggle MCP');
			console.error(err);
		}
	}

	// Skill handlers
	async function handleAddSkill(skill: Skill) {
		try {
			await skillLibrary.assignToProject(project.id, skill.id);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Added ${skill.name} to ${project.name}`);
		} catch (err) {
			notifications.error('Failed to add skill');
			console.error(err);
		}
	}

	async function handleRemoveSkill(skillId: number) {
		try {
			const skill = skillLibrary.getSkillById(skillId);
			await skillLibrary.removeFromProject(project.id, skillId);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Removed ${skill?.name || 'Skill'} from ${project.name}`);
		} catch (err) {
			notifications.error('Failed to remove skill');
			console.error(err);
		}
	}

	async function handleToggleSkill(assignmentId: number, enabled: boolean) {
		try {
			await skillLibrary.toggleProjectSkill(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
		} catch (err) {
			notifications.error('Failed to toggle skill');
			console.error(err);
		}
	}

	// SubAgent handlers
	async function handleAddSubAgent(agent: SubAgent) {
		try {
			await subagentLibrary.assignToProject(project.id, agent.id);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Added ${agent.name} to ${project.name}`);
		} catch (err) {
			notifications.error('Failed to add agent');
			console.error(err);
		}
	}

	async function handleRemoveSubAgent(agentId: number) {
		try {
			const agent = subagentLibrary.getSubAgentById(agentId);
			await subagentLibrary.removeFromProject(project.id, agentId);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Removed ${agent?.name || 'Agent'} from ${project.name}`);
		} catch (err) {
			notifications.error('Failed to remove agent');
			console.error(err);
		}
	}

	async function handleToggleSubAgent(assignmentId: number, enabled: boolean) {
		try {
			await subagentLibrary.toggleProjectSubAgent(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
		} catch (err) {
			notifications.error('Failed to toggle agent');
			console.error(err);
		}
	}

	// Command handlers
	async function handleAddCommand(command: Command) {
		try {
			await commandLibrary.assignToProject(project.id, command.id);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Added ${command.name} to ${project.name}`);
		} catch (err) {
			notifications.error('Failed to add command');
			console.error(err);
		}
	}

	async function handleRemoveCommand(commandId: number) {
		try {
			const command = commandLibrary.getCommandById(commandId);
			await commandLibrary.removeFromProject(project.id, commandId);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
			notifications.success(`Removed ${command?.name || 'Command'} from ${project.name}`);
		} catch (err) {
			notifications.error('Failed to remove command');
			console.error(err);
		}
	}

	async function handleToggleCommand(assignmentId: number, enabled: boolean) {
		try {
			await commandLibrary.toggleProjectCommand(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
			await loadProjectData();
		} catch (err) {
			notifications.error('Failed to toggle command');
			console.error(err);
		}
	}

	// Editor type handler
	async function handleChangeEditorType(editorType: 'claude_code' | 'opencode') {
		if (editorType === project.editorType) {
			showEditorDropdown = false;
			return;
		}
		updatingEditor = true;
		try {
			await invoke('update_project_editor_type', { projectId: project.id, editorType });
			await projectsStore.loadProjects();
			notifications.success(`Switched to ${editorType === 'claude_code' ? 'Claude Code' : 'OpenCode'}`);
		} catch (err) {
			notifications.error('Failed to change editor');
			console.error(err);
		} finally {
			updatingEditor = false;
			showEditorDropdown = false;
		}
	}

	function getEditorDisplayName(editorType: string): string {
		return editorType === 'claude_code' ? 'Claude Code' : 'OpenCode';
	}

	// Clear search when switching tabs
	function handleTabChange(tab: Tab) {
		searchQuery = '';
		activeTab = tab;
	}

	// Get search placeholder based on active tab
	function getSearchPlaceholder(): string {
		switch (activeTab) {
			case 'mcps': return 'Search available MCPs...';
			case 'skills': return 'Search available skills...';
			case 'agents': return 'Search available agents...';
			case 'commands': return 'Search available commands...';
		}
	}

</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onclick={onClose}
	role="dialog"
	aria-modal="true"
	aria-labelledby="project-detail-title"
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[85vh] flex flex-col"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
					<FolderOpen class="w-5 h-5 text-amber-600 dark:text-amber-400" />
				</div>
				<div>
					<div class="flex items-center gap-2">
						<h2 id="project-detail-title" class="text-xl font-semibold text-gray-900 dark:text-white">{project.name}</h2>
						<!-- Editor Type Dropdown -->
						<div class="relative">
							<button
								onclick={(e) => { e.stopPropagation(); showEditorDropdown = !showEditorDropdown; }}
								disabled={updatingEditor}
								class="inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium transition-colors {project.editorType === 'opencode'
									? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300 hover:bg-emerald-200 dark:hover:bg-emerald-900/70'
									: 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300 hover:bg-primary-200 dark:hover:bg-primary-900/70'}"
							>
								<span class="w-3 h-3 rounded-sm flex items-center justify-center text-[8px] font-bold text-white {project.editorType === 'opencode' ? 'bg-emerald-500' : 'bg-primary-500'}">
									{project.editorType === 'opencode' ? 'O' : 'C'}
								</span>
								{getEditorDisplayName(project.editorType)}
								<ChevronDown class="w-3 h-3" />
							</button>
							{#if showEditorDropdown}
								<div class="absolute left-0 top-full mt-1 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-20">
									<button
										onclick={() => handleChangeEditorType('claude_code')}
										class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 {project.editorType === 'claude_code' ? 'text-primary-600 dark:text-primary-400' : 'text-gray-700 dark:text-gray-300'}"
									>
										<span class="w-4 h-4 rounded-sm bg-primary-500 text-white flex items-center justify-center text-[9px] font-bold">C</span>
										Claude Code
									</button>
									<button
										onclick={() => handleChangeEditorType('opencode')}
										class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 {project.editorType === 'opencode' ? 'text-emerald-600 dark:text-emerald-400' : 'text-gray-700 dark:text-gray-300'}"
									>
										<span class="w-4 h-4 rounded-sm bg-emerald-500 text-white flex items-center justify-center text-[9px] font-bold">O</span>
										OpenCode
									</button>
								</div>
							{/if}
						</div>
					</div>
					<p class="text-sm text-gray-500 dark:text-gray-400 font-mono">{project.path}</p>
				</div>
			</div>
			<button
				onclick={onClose}
				class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Tabs -->
		<div class="flex border-b border-gray-200 dark:border-gray-700 px-6">
			<button
				onclick={() => handleTabChange('mcps')}
				class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'mcps' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
			>
				<Plug class="w-4 h-4" />
				MCPs ({project.assignedMcps.length})
			</button>
			<button
				onclick={() => handleTabChange('skills')}
				class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'skills' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
			>
				<Sparkles class="w-4 h-4" />
				Skills ({projectSkills.length})
			</button>
			<button
				onclick={() => handleTabChange('agents')}
				class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'agents' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
			>
				<Bot class="w-4 h-4" />
				Agents ({projectSubAgents.length})
			</button>
			<button
				onclick={() => handleTabChange('commands')}
				class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'commands' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
			>
				<Terminal class="w-4 h-4" />
				Commands ({projectCommands.length})
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-6 space-y-6">
			{#if activeTab === 'mcps'}
				<!-- Assigned MCPs -->
				<div>
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
						Assigned MCPs ({project.assignedMcps.length})
					</h3>
					{#if project.assignedMcps.length > 0}
						<div class="space-y-2">
							{#each project.assignedMcps as assignment (assignment.id)}
								{@const mcp = mcpLibrary.getMcpById(assignment.mcpId) ?? assignment.mcp}
								<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
									<div class="flex items-center gap-3">
										<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
											<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
										</div>
										<div>
											<span class="font-medium text-gray-900 dark:text-white {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
												{mcp.name}
											</span>
											<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
										</div>
									</div>
									<div class="flex items-center gap-3">
										<!-- Toggle -->
										<button
											onclick={() => handleToggleMcp(assignment.id, !assignment.isEnabled)}
											class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
											role="switch"
											aria-checked={assignment.isEnabled}
											title={assignment.isEnabled ? 'Disable' : 'Enable'}
										>
											<span
												class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
											></span>
										</button>
										<!-- Remove -->
										<button
											onclick={() => handleRemoveMcp(assignment.mcpId)}
											class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
											title="Remove from project"
										>
											<Minus class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">No MCPs assigned yet</p>
							<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add MCPs from the library below</p>
						</div>
					{/if}
				</div>

				<!-- Available MCPs -->
				<div>
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Available MCPs ({availableMcps.length})
						</h3>
						{#if availableMcps.length > 3}
							<div class="relative w-48">
								<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={getSearchPlaceholder()}
									class="input pl-8 py-1.5 text-sm"
								/>
							</div>
						{/if}
					</div>
					{#if availableMcps.length > 0}
						{#if filteredAvailableMcps.length > 0}
							<div class="space-y-2">
								{#each filteredAvailableMcps as mcp (mcp.id)}
									<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
										<div class="flex items-center gap-3">
											<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
												<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
											</div>
											<div>
												<span class="font-medium text-gray-900 dark:text-white">{mcp.name}</span>
												<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
											</div>
										</div>
										<button
											onclick={() => handleAddMcp(mcp)}
											class="p-1.5 text-gray-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors"
											title="Add to project"
										>
											<Plus class="w-4 h-4" />
										</button>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-4 text-sm text-gray-500 dark:text-gray-400">
								No MCPs match "{searchQuery}"
							</div>
						{/if}
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">All MCPs are assigned</p>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'skills'}
				<!-- Assigned Skills -->
				<div>
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
						Assigned Skills ({projectSkills.length})
					</h3>
					{#if projectSkills.length > 0}
						<div class="space-y-2">
							{#each projectSkills as assignment (assignment.id)}
								{@const skill = skillLibrary.getSkillById(assignment.skillId) ?? assignment.skill}
								<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
									<div class="flex items-center gap-3 min-w-0 flex-1">
										<div class="w-8 h-8 rounded-lg bg-yellow-100 text-yellow-600 dark:bg-yellow-900/50 dark:text-yellow-400 flex items-center justify-center flex-shrink-0">
											<Sparkles class="w-4 h-4" />
										</div>
										<div class="min-w-0">
											<p class="font-medium text-gray-900 dark:text-white truncate {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
												{skill.name}
											</p>
											{#if skill.description}
												<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{skill.description}</p>
											{/if}
										</div>
									</div>
									<div class="flex items-center gap-3 flex-shrink-0">
										<!-- Toggle -->
										<button
											onclick={() => handleToggleSkill(assignment.id, !assignment.isEnabled)}
											class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
											role="switch"
											aria-checked={assignment.isEnabled}
											title={assignment.isEnabled ? 'Disable' : 'Enable'}
										>
											<span
												class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
											></span>
										</button>
										<!-- Remove -->
										<button
											onclick={() => handleRemoveSkill(assignment.skillId)}
											class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
											title="Remove from project"
										>
											<Minus class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">No skills assigned yet</p>
							<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add skills from the library below</p>
						</div>
					{/if}
				</div>

				<!-- Available Skills -->
				<div>
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Available Skills ({availableSkills.length})
						</h3>
						{#if availableSkills.length > 3}
							<div class="relative w-48">
								<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={getSearchPlaceholder()}
									class="input pl-8 py-1.5 text-sm"
								/>
							</div>
						{/if}
					</div>
					{#if availableSkills.length > 0}
						{#if filteredAvailableSkills.length > 0}
							<div class="space-y-2">
								{#each filteredAvailableSkills as skill (skill.id)}
									<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
										<div class="flex items-center gap-3 min-w-0 flex-1">
											<div class="w-8 h-8 rounded-lg bg-yellow-100 text-yellow-600 dark:bg-yellow-900/50 dark:text-yellow-400 flex items-center justify-center flex-shrink-0">
												<Sparkles class="w-4 h-4" />
											</div>
											<div class="min-w-0">
												<p class="font-medium text-gray-900 dark:text-white truncate">{skill.name}</p>
												{#if skill.description}
													<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{skill.description}</p>
												{/if}
											</div>
										</div>
										<button
											onclick={() => handleAddSkill(skill)}
											class="p-1.5 text-gray-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors flex-shrink-0"
											title="Add to project"
										>
											<Plus class="w-4 h-4" />
										</button>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-4 text-sm text-gray-500 dark:text-gray-400">
								No skills match "{searchQuery}"
							</div>
						{/if}
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">All skills are assigned</p>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'agents'}
				<!-- Assigned SubAgents -->
				<div>
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
						Assigned Agents ({projectSubAgents.length})
					</h3>
					{#if projectSubAgents.length > 0}
						<div class="space-y-2">
							{#each projectSubAgents as assignment (assignment.id)}
								{@const agent = subagentLibrary.getSubAgentById(assignment.subagentId) ?? assignment.subagent}
								<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
									<div class="flex items-center gap-3 min-w-0 flex-1">
										<div class="w-8 h-8 rounded-lg bg-cyan-100 text-cyan-600 dark:bg-cyan-900/50 dark:text-cyan-400 flex items-center justify-center flex-shrink-0">
											<Bot class="w-4 h-4" />
										</div>
										<div class="min-w-0">
											<p class="font-medium text-gray-900 dark:text-white truncate {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
												{agent.name}
											</p>
											{#if agent.description}
												<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{agent.description}</p>
											{/if}
										</div>
									</div>
									<div class="flex items-center gap-3 flex-shrink-0">
										<!-- Toggle -->
										<button
											onclick={() => handleToggleSubAgent(assignment.id, !assignment.isEnabled)}
											class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
											role="switch"
											aria-checked={assignment.isEnabled}
											title={assignment.isEnabled ? 'Disable' : 'Enable'}
										>
											<span
												class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
											></span>
										</button>
										<!-- Remove -->
										<button
											onclick={() => handleRemoveSubAgent(assignment.subagentId)}
											class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
											title="Remove from project"
										>
											<Minus class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">No agents assigned yet</p>
							<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add agents from the library below</p>
						</div>
					{/if}
				</div>

				<!-- Available SubAgents -->
				<div>
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Available Agents ({availableSubAgents.length})
						</h3>
						{#if availableSubAgents.length > 3}
							<div class="relative w-48">
								<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={getSearchPlaceholder()}
									class="input pl-8 py-1.5 text-sm"
								/>
							</div>
						{/if}
					</div>
					{#if availableSubAgents.length > 0}
						{#if filteredAvailableSubAgents.length > 0}
							<div class="space-y-2">
								{#each filteredAvailableSubAgents as agent (agent.id)}
									<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
										<div class="flex items-center gap-3 min-w-0 flex-1">
											<div class="w-8 h-8 rounded-lg bg-cyan-100 text-cyan-600 dark:bg-cyan-900/50 dark:text-cyan-400 flex items-center justify-center flex-shrink-0">
												<Bot class="w-4 h-4" />
											</div>
											<div class="min-w-0">
												<p class="font-medium text-gray-900 dark:text-white truncate">{agent.name}</p>
												{#if agent.description}
													<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{agent.description}</p>
												{/if}
											</div>
										</div>
										<button
											onclick={() => handleAddSubAgent(agent)}
											class="p-1.5 text-gray-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors flex-shrink-0"
											title="Add to project"
										>
											<Plus class="w-4 h-4" />
										</button>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-4 text-sm text-gray-500 dark:text-gray-400">
								No agents match "{searchQuery}"
							</div>
						{/if}
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">All agents are assigned</p>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'commands'}
				<!-- Assigned Commands -->
				<div>
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
						Assigned Commands ({projectCommands.length})
					</h3>
					{#if projectCommands.length > 0}
						<div class="space-y-2">
							{#each projectCommands as assignment (assignment.id)}
								{@const command = commandLibrary.getCommandById(assignment.commandId) ?? assignment.command}
								<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
									<div class="flex items-center gap-3 min-w-0 flex-1">
										<div class="w-8 h-8 rounded-lg bg-indigo-100 text-indigo-600 dark:bg-indigo-900/50 dark:text-indigo-400 flex items-center justify-center flex-shrink-0">
											<Terminal class="w-4 h-4" />
										</div>
										<div class="min-w-0">
											<p class="font-medium text-gray-900 dark:text-white truncate {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
												/{command.name}
											</p>
											{#if command.description}
												<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{command.description}</p>
											{/if}
										</div>
									</div>
									<div class="flex items-center gap-3 flex-shrink-0">
										<!-- Toggle -->
										<button
											onclick={() => handleToggleCommand(assignment.id, !assignment.isEnabled)}
											class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
											role="switch"
											aria-checked={assignment.isEnabled}
											title={assignment.isEnabled ? 'Disable' : 'Enable'}
										>
											<span
												class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
											></span>
										</button>
										<!-- Remove -->
										<button
											onclick={() => handleRemoveCommand(assignment.commandId)}
											class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
											title="Remove from project"
										>
											<Minus class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/each}
						</div>
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">No commands assigned yet</p>
							<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add commands from the library below</p>
						</div>
					{/if}
				</div>

				<!-- Available Commands -->
				<div>
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Available Commands ({availableCommands.length})
						</h3>
						{#if availableCommands.length > 3}
							<div class="relative w-48">
								<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={getSearchPlaceholder()}
									class="input pl-8 py-1.5 text-sm"
								/>
							</div>
						{/if}
					</div>
					{#if availableCommands.length > 0}
						{#if filteredAvailableCommands.length > 0}
							<div class="space-y-2">
								{#each filteredAvailableCommands as command (command.id)}
									<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
										<div class="flex items-center gap-3 min-w-0 flex-1">
											<div class="w-8 h-8 rounded-lg bg-indigo-100 text-indigo-600 dark:bg-indigo-900/50 dark:text-indigo-400 flex items-center justify-center flex-shrink-0">
												<Terminal class="w-4 h-4" />
											</div>
											<div class="min-w-0">
												<p class="font-medium text-gray-900 dark:text-white truncate">/{command.name}</p>
												{#if command.description}
													<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{command.description}</p>
												{/if}
											</div>
										</div>
										<button
											onclick={() => handleAddCommand(command)}
											class="p-1.5 text-gray-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors flex-shrink-0"
											title="Add to project"
										>
											<Plus class="w-4 h-4" />
										</button>
									</div>
								{/each}
							</div>
						{:else}
							<div class="text-center py-4 text-sm text-gray-500 dark:text-gray-400">
								No commands match "{searchQuery}"
							</div>
						{/if}
					{:else}
						<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
							<p class="text-gray-500 dark:text-gray-400">All commands are assigned</p>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>
