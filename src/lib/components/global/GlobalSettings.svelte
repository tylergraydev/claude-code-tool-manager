<script lang="ts">
	import type { Mcp, Skill, SubAgent, Command, GlobalSkill, GlobalSubAgent, GlobalCommand } from '$lib/types';
	import { projectsStore, notifications, mcpLibrary, skillLibrary, subagentLibrary, commandLibrary, debugStore } from '$lib/stores';
	import { Globe, RefreshCw, Plus, Minus, Plug, Server, Sparkles, Bot, Bug, FolderOpen, Loader2, Terminal } from 'lucide-svelte';
	import { installDebugInterceptor, uninstallDebugInterceptor } from '$lib/utils/debugLogger';
	import { onMount } from 'svelte';

	// Load debug state on mount
	onMount(() => {
		debugStore.load();
	});

	async function handleToggleDebug() {
		try {
			if (debugStore.isEnabled) {
				await debugStore.disable();
				uninstallDebugInterceptor();
				notifications.success('Debug mode disabled');
			} else {
				await debugStore.enable();
				installDebugInterceptor();
				notifications.success('Debug mode enabled');
			}
		} catch (e) {
			notifications.error('Failed to toggle debug mode');
		}
	}

	async function handleOpenLogsFolder() {
		try {
			await debugStore.openLogsFolder();
		} catch {
			notifications.error('Failed to open logs folder');
		}
	}

	// Tab state
	type Tab = 'mcps' | 'commands' | 'skills' | 'agents';
	let activeTab = $state<Tab>('mcps');

	let showAddModal = $state(false);

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

	// MCP state
	let globalMcpIds = $derived(projectsStore.globalMcps.map((g) => g.mcpId));
	let availableMcps = $derived(
		mcpLibrary.mcps.filter((mcp) => !globalMcpIds.includes(mcp.id))
	);

	// Commands state
	let globalCommandIds = $derived(commandLibrary.globalCommands.map((g) => g.commandId));
	let availableCommands = $derived(
		commandLibrary.commands.filter((cmd) => !globalCommandIds.includes(cmd.id))
	);

	// Skills state
	let globalSkillIds = $derived(skillLibrary.globalSkills.map((g) => g.skillId));
	let availableSkills = $derived(
		skillLibrary.skills.filter((skill) => !globalSkillIds.includes(skill.id))
	);

	// Agents state
	let globalAgentIds = $derived(subagentLibrary.globalSubAgents.map((g) => g.subagentId));
	let availableAgents = $derived(
		subagentLibrary.subagents.filter((agent) => !globalAgentIds.includes(agent.id))
	);

	// Load global commands, skills and agents on mount
	$effect(() => {
		commandLibrary.loadGlobalCommands();
		skillLibrary.loadGlobalSkills();
		subagentLibrary.loadGlobalSubAgents();
	});

	async function handleSync() {
		try {
			await projectsStore.syncGlobalConfig();
			notifications.success('Global config synced');
		} catch {
			notifications.error('Failed to sync config');
		}
	}

	// MCP handlers
	async function handleAddMcp(mcp: Mcp) {
		try {
			await projectsStore.addGlobalMcp(mcp.id);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Added ${mcp.name} to global settings`);
		} catch {
			notifications.error('Failed to add MCP');
		}
	}

	async function handleRemoveMcp(mcpId: number) {
		try {
			const mcp = mcpLibrary.getMcpById(mcpId);
			await projectsStore.removeGlobalMcp(mcpId);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Removed ${mcp?.name || 'MCP'} from global settings`);
		} catch {
			notifications.error('Failed to remove MCP');
		}
	}

	async function handleToggleMcp(assignmentId: number, enabled: boolean) {
		try {
			await projectsStore.toggleGlobalMcp(assignmentId, enabled);
			await projectsStore.syncGlobalConfig();
		} catch {
			notifications.error('Failed to toggle MCP');
		}
	}

	// Command handlers
	async function handleAddCommand(command: Command) {
		try {
			await commandLibrary.addGlobalCommand(command.id);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Added ${command.name} to global settings`);
		} catch {
			notifications.error('Failed to add command');
		}
	}

	async function handleRemoveCommand(commandId: number) {
		try {
			const command = commandLibrary.getCommandById(commandId);
			await commandLibrary.removeGlobalCommand(commandId);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Removed ${command?.name || 'Command'} from global settings`);
		} catch {
			notifications.error('Failed to remove command');
		}
	}

	async function handleToggleCommand(assignmentId: number, enabled: boolean) {
		try {
			await commandLibrary.toggleGlobalCommand(assignmentId, enabled);
			await projectsStore.syncGlobalConfig();
		} catch {
			notifications.error('Failed to toggle command');
		}
	}

	// Skill handlers
	async function handleAddSkill(skill: Skill) {
		try {
			await skillLibrary.addGlobalSkill(skill.id);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Added ${skill.name} to global settings`);
		} catch {
			notifications.error('Failed to add skill');
		}
	}

	async function handleRemoveSkill(skillId: number) {
		try {
			const skill = skillLibrary.getSkillById(skillId);
			await skillLibrary.removeGlobalSkill(skillId);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Removed ${skill?.name || 'Skill'} from global settings`);
		} catch {
			notifications.error('Failed to remove skill');
		}
	}

	async function handleToggleSkill(assignmentId: number, enabled: boolean) {
		try {
			await skillLibrary.toggleGlobalSkill(assignmentId, enabled);
			await projectsStore.syncGlobalConfig();
		} catch {
			notifications.error('Failed to toggle skill');
		}
	}

	// Agent handlers
	async function handleAddAgent(agent: SubAgent) {
		try {
			await subagentLibrary.addGlobalSubAgent(agent.id);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Added ${agent.name} to global settings`);
		} catch {
			notifications.error('Failed to add agent');
		}
	}

	async function handleRemoveAgent(agentId: number) {
		try {
			const agent = subagentLibrary.getSubAgentById(agentId);
			await subagentLibrary.removeGlobalSubAgent(agentId);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Removed ${agent?.name || 'Agent'} from global settings`);
		} catch {
			notifications.error('Failed to remove agent');
		}
	}

	async function handleToggleAgent(assignmentId: number, enabled: boolean) {
		try {
			await subagentLibrary.toggleGlobalSubAgent(assignmentId, enabled);
			await projectsStore.syncGlobalConfig();
		} catch {
			notifications.error('Failed to toggle agent');
		}
	}

	function getAddButtonLabel() {
		switch (activeTab) {
			case 'mcps': return 'Add MCP';
			case 'commands': return 'Add Command';
			case 'skills': return 'Add Skill';
			case 'agents': return 'Add Agent';
		}
	}

	function getAddModalTitle() {
		switch (activeTab) {
			case 'mcps': return 'Add Global MCP';
			case 'commands': return 'Add Global Command';
			case 'skills': return 'Add Global Skill';
			case 'agents': return 'Add Global Agent';
		}
	}
</script>

<div class="space-y-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 rounded-xl bg-indigo-100 dark:bg-indigo-900/50 flex items-center justify-center">
				<Globe class="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
			</div>
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Global Settings</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400">
					Available in all projects
				</p>
			</div>
		</div>
		<div class="flex gap-2">
			<button onclick={() => (showAddModal = true)} class="btn btn-primary">
				<Plus class="w-4 h-4 mr-2" />
				{getAddButtonLabel()}
			</button>
			<button onclick={handleSync} class="btn btn-secondary">
				<RefreshCw class="w-4 h-4 mr-2" />
				Sync
			</button>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex border-b border-gray-200 dark:border-gray-700">
		<button
			onclick={() => activeTab = 'mcps'}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'mcps' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Plug class="w-4 h-4" />
			MCPs ({projectsStore.globalMcps.length})
		</button>
		<button
			onclick={() => activeTab = 'commands'}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'commands' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Terminal class="w-4 h-4" />
			Commands ({commandLibrary.globalCommands.length})
		</button>
		<button
			onclick={() => activeTab = 'skills'}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'skills' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Sparkles class="w-4 h-4" />
			Skills ({skillLibrary.globalSkills.length})
		</button>
		<button
			onclick={() => activeTab = 'agents'}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'agents' ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Bot class="w-4 h-4" />
			Agents ({subagentLibrary.globalSubAgents.length})
		</button>
	</div>

	<!-- Content -->
	<div class="card">
		{#if activeTab === 'mcps'}
			{#if projectsStore.globalMcps.length > 0}
				<div class="space-y-2">
					{#each projectsStore.globalMcps as assignment (assignment.id)}
						{@const mcp = mcpLibrary.getMcpById(assignment.mcpId) ?? assignment.mcp}
						<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
							<div class="flex items-center gap-3 min-w-0 flex-1">
								<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center flex-shrink-0">
									<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
								</div>
								<div class="min-w-0">
									<p class="font-medium text-gray-900 dark:text-white truncate {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
										{mcp.name}
									</p>
									<p class="text-xs text-gray-500 dark:text-gray-400 truncate">({mcp.type})</p>
								</div>
							</div>
							<div class="flex items-center gap-3 flex-shrink-0">
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
								<button
									onclick={() => handleRemoveMcp(assignment.mcpId)}
									class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
									title="Remove"
								>
									<Minus class="w-4 h-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="text-center py-8">
					<Plug class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No global MCPs</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
						Add MCPs to make them available in all projects
					</p>
					<button onclick={() => (showAddModal = true)} class="btn btn-primary">
						<Plus class="w-4 h-4 mr-2" />
						Add MCP
					</button>
				</div>
			{/if}
		{:else if activeTab === 'commands'}
			{#if commandLibrary.globalCommands.length > 0}
				<div class="space-y-2">
					{#each commandLibrary.globalCommands as assignment (assignment.id)}
						{@const command = commandLibrary.getCommandById(assignment.commandId) ?? assignment.command}
						<div class="flex items-center justify-between gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
							<div class="flex items-center gap-3 min-w-0 flex-1">
								<div class="w-8 h-8 rounded-lg bg-amber-100 text-amber-600 dark:bg-amber-900/50 dark:text-amber-400 flex items-center justify-center flex-shrink-0">
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
								<button
									onclick={() => handleRemoveCommand(assignment.commandId)}
									class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
									title="Remove"
								>
									<Minus class="w-4 h-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="text-center py-8">
					<Terminal class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No global commands</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
						Add commands to make them available in all projects
					</p>
					<button onclick={() => (showAddModal = true)} class="btn btn-primary">
						<Plus class="w-4 h-4 mr-2" />
						Add Command
					</button>
				</div>
			{/if}
		{:else if activeTab === 'skills'}
			{#if skillLibrary.globalSkills.length > 0}
				<div class="space-y-2">
					{#each skillLibrary.globalSkills as assignment (assignment.id)}
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
								<button
									onclick={() => handleRemoveSkill(assignment.skillId)}
									class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
									title="Remove"
								>
									<Minus class="w-4 h-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="text-center py-8">
					<Sparkles class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No global skills</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
						Add skills to make them available in all projects
					</p>
					<button onclick={() => (showAddModal = true)} class="btn btn-primary">
						<Plus class="w-4 h-4 mr-2" />
						Add Skill
					</button>
				</div>
			{/if}
		{:else if activeTab === 'agents'}
			{#if subagentLibrary.globalSubAgents.length > 0}
				<div class="space-y-2">
					{#each subagentLibrary.globalSubAgents as assignment (assignment.id)}
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
								<button
									onclick={() => handleToggleAgent(assignment.id, !assignment.isEnabled)}
									class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
									role="switch"
									aria-checked={assignment.isEnabled}
									title={assignment.isEnabled ? 'Disable' : 'Enable'}
								>
									<span
										class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
									></span>
								</button>
								<button
									onclick={() => handleRemoveAgent(assignment.subagentId)}
									class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
									title="Remove"
								>
									<Minus class="w-4 h-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="text-center py-8">
					<Bot class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No global agents</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
						Add agents to make them available in all projects
					</p>
					<button onclick={() => (showAddModal = true)} class="btn btn-primary">
						<Plus class="w-4 h-4 mr-2" />
						Add Agent
					</button>
				</div>
			{/if}
		{/if}
	</div>

	<!-- Debug Mode Section -->
	<div class="card p-6 mt-6">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-xl bg-orange-100 dark:bg-orange-900/50 flex items-center justify-center">
					<Bug class="w-5 h-5 text-orange-600 dark:text-orange-400" />
				</div>
				<div>
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Debug Mode</h2>
					<p class="text-sm text-gray-500 dark:text-gray-400">
						Enable logging to help troubleshoot issues
					</p>
				</div>
			</div>
			<button
				onclick={handleToggleDebug}
				disabled={debugStore.isLoading}
				class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none disabled:opacity-50 {debugStore.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
				role="switch"
				aria-checked={debugStore.isEnabled}
				title={debugStore.isEnabled ? 'Disable debug mode' : 'Enable debug mode'}
			>
				{#if debugStore.isLoading}
					<span class="absolute inset-0 flex items-center justify-center">
						<Loader2 class="w-4 h-4 animate-spin text-white" />
					</span>
				{:else}
					<span
						class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {debugStore.isEnabled ? 'translate-x-5' : 'translate-x-0'}"
					></span>
				{/if}
			</button>
		</div>

		{#if debugStore.isEnabled || debugStore.logFilePath}
			<div class="mt-4 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
				<div class="flex items-center justify-between gap-4">
					<div class="min-w-0 flex-1">
						<p class="text-sm font-medium text-gray-700 dark:text-gray-300">Log file location:</p>
						<code class="text-xs text-gray-500 dark:text-gray-400 break-all">
							{debugStore.logFilePath || 'No active log file'}
						</code>
					</div>
					<button
						onclick={handleOpenLogsFolder}
						class="btn btn-secondary flex-shrink-0"
					>
						<FolderOpen class="w-4 h-4 mr-2" />
						Open Folder
					</button>
				</div>
				{#if debugStore.isEnabled}
					<p class="mt-3 text-xs text-gray-500 dark:text-gray-400">
						Debug mode is active. Logs are being written to the file above. Share this file when reporting issues.
					</p>
				{/if}
			</div>
		{/if}
	</div>
</div>

<!-- Add Modal -->
{#if showAddModal}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={() => (showAddModal = false)}>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 max-h-[70vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">{getAddModalTitle()}</h3>
				<button
					onclick={() => (showAddModal = false)}
					class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl"
				>
					&times;
				</button>
			</div>
			<div class="flex-1 overflow-auto p-4">
				{#if activeTab === 'mcps'}
					{#if availableMcps.length > 0}
						<div class="space-y-2">
							{#each availableMcps as mcp (mcp.id)}
								<button
									onclick={() => {
										handleAddMcp(mcp);
										showAddModal = false;
									}}
									class="w-full flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
								>
									<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center flex-shrink-0">
										<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
									</div>
									<div class="flex-1 min-w-0">
										<p class="font-medium text-gray-900 dark:text-white truncate">{mcp.name}</p>
										<p class="text-xs text-gray-500 dark:text-gray-400 truncate">({mcp.type})</p>
									</div>
									<Plus class="w-4 h-4 text-gray-400 flex-shrink-0" />
								</button>
							{/each}
						</div>
					{:else}
						<div class="text-center py-8 text-gray-500 dark:text-gray-400">
							All MCPs are already in global settings
						</div>
					{/if}
				{:else if activeTab === 'commands'}
					{#if availableCommands.length > 0}
						<div class="space-y-2">
							{#each availableCommands as command (command.id)}
								<button
									onclick={() => {
										handleAddCommand(command);
										showAddModal = false;
									}}
									class="w-full flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
								>
									<div class="w-8 h-8 rounded-lg bg-amber-100 text-amber-600 dark:bg-amber-900/50 dark:text-amber-400 flex items-center justify-center flex-shrink-0">
										<Terminal class="w-4 h-4" />
									</div>
									<div class="flex-1 min-w-0">
										<p class="font-medium text-gray-900 dark:text-white truncate">/{command.name}</p>
										{#if command.description}
											<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{command.description}</p>
										{/if}
									</div>
									<Plus class="w-4 h-4 text-gray-400 flex-shrink-0" />
								</button>
							{/each}
						</div>
					{:else}
						<div class="text-center py-8 text-gray-500 dark:text-gray-400">
							All commands are already in global settings
						</div>
					{/if}
				{:else if activeTab === 'skills'}
					{#if availableSkills.length > 0}
						<div class="space-y-2">
							{#each availableSkills as skill (skill.id)}
								<button
									onclick={() => {
										handleAddSkill(skill);
										showAddModal = false;
									}}
									class="w-full flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
								>
									<div class="w-8 h-8 rounded-lg bg-yellow-100 text-yellow-600 dark:bg-yellow-900/50 dark:text-yellow-400 flex items-center justify-center flex-shrink-0">
										<Sparkles class="w-4 h-4" />
									</div>
									<div class="flex-1 min-w-0">
										<p class="font-medium text-gray-900 dark:text-white truncate">{skill.name}</p>
										{#if skill.description}
											<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{skill.description}</p>
										{/if}
									</div>
									<Plus class="w-4 h-4 text-gray-400 flex-shrink-0" />
								</button>
							{/each}
						</div>
					{:else}
						<div class="text-center py-8 text-gray-500 dark:text-gray-400">
							All skills are already in global settings
						</div>
					{/if}
				{:else if activeTab === 'agents'}
					{#if availableAgents.length > 0}
						<div class="space-y-2">
							{#each availableAgents as agent (agent.id)}
								<button
									onclick={() => {
										handleAddAgent(agent);
										showAddModal = false;
									}}
									class="w-full flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
								>
									<div class="w-8 h-8 rounded-lg bg-cyan-100 text-cyan-600 dark:bg-cyan-900/50 dark:text-cyan-400 flex items-center justify-center flex-shrink-0">
										<Bot class="w-4 h-4" />
									</div>
									<div class="flex-1 min-w-0">
										<p class="font-medium text-gray-900 dark:text-white truncate">{agent.name}</p>
										{#if agent.description}
											<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{agent.description}</p>
										{/if}
									</div>
									<Plus class="w-4 h-4 text-gray-400 flex-shrink-0" />
								</button>
							{/each}
						</div>
					{:else}
						<div class="text-center py-8 text-gray-500 dark:text-gray-400">
							All agents are already in global settings
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</div>
{/if}
