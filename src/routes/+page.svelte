<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { McpLibrary, McpForm } from '$lib/components/mcp';
	import { ProjectList } from '$lib/components/projects';
	import { GlobalSettings } from '$lib/components/global';
	import { ConfirmDialog } from '$lib/components/shared';
	import { mcpLibrary, projectsStore, notifications } from '$lib/stores';
	import type { Mcp, Project } from '$lib/types';
	import { invoke } from '@tauri-apps/api/core';
	import { Plus, Scan } from 'lucide-svelte';

	// Modal states
	let showAddMcp = $state(false);
	let editingMcp = $state<Mcp | null>(null);
	let deletingMcp = $state<Mcp | null>(null);
	let deletingProject = $state<Project | null>(null);

	async function handleCreateMcp(values: any) {
		try {
			await mcpLibrary.create(values);
			showAddMcp = false;
			notifications.success('MCP created successfully');
		} catch (err) {
			notifications.error('Failed to create MCP');
		}
	}

	async function handleUpdateMcp(values: any) {
		if (!editingMcp) return;
		try {
			await mcpLibrary.update(editingMcp.id, values);
			editingMcp = null;
			notifications.success('MCP updated successfully');
		} catch (err) {
			notifications.error('Failed to update MCP');
		}
	}

	async function handleDeleteMcp() {
		if (!deletingMcp) return;
		try {
			await mcpLibrary.delete(deletingMcp.id);
			notifications.success('MCP deleted');
		} catch (err) {
			notifications.error('Failed to delete MCP');
		} finally {
			deletingMcp = null;
		}
	}

	async function handleDuplicateMcp(mcp: Mcp) {
		try {
			await mcpLibrary.duplicate(mcp.id);
			notifications.success('MCP duplicated');
		} catch (err) {
			notifications.error('Failed to duplicate MCP');
		}
	}

	async function handleAddProject() {
		try {
			const path = await projectsStore.browseForProject();
			if (path) {
				const name = path.split(/[/\\]/).pop() || 'Project';
				await projectsStore.addProject({ name, path });
				notifications.success('Project added');
			}
		} catch (err) {
			notifications.error('Failed to add project');
		}
	}

	async function handleRemoveProject() {
		if (!deletingProject) return;
		try {
			await projectsStore.removeProject(deletingProject.id);
			notifications.success('Project removed');
		} catch (err) {
			notifications.error('Failed to remove project');
		} finally {
			deletingProject = null;
		}
	}

	async function handleScan() {
		try {
			notifications.info('Scanning for MCPs...');
			await invoke('scan_claude_directory');
			await mcpLibrary.load();
			notifications.success('Scan complete');
		} catch (err) {
			notifications.error('Scan failed');
		}
	}
</script>

<Header title="Dashboard" subtitle="Manage your Claude Code MCPs, Agents, and Skills" />

<div class="flex-1 overflow-auto p-6 space-y-8">
	<!-- Quick Actions -->
	<div class="flex items-center gap-3">
		<button onclick={() => (showAddMcp = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add MCP
		</button>
		<button onclick={handleScan} class="btn btn-secondary">
			<Scan class="w-4 h-4 mr-2" />
			Scan for MCPs
		</button>
	</div>

	<!-- MCP Library -->
	<section>
		<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">MCP Library</h2>
		<McpLibrary
			onEdit={(mcp) => (editingMcp = mcp)}
			onDelete={(mcp) => (deletingMcp = mcp)}
			onDuplicate={handleDuplicateMcp}
		/>
	</section>

	<!-- Projects Section -->
	<section>
		<ProjectList
			onAddProject={handleAddProject}
			onRemoveProject={(project) => (deletingProject = project)}
		/>
	</section>

	<!-- Global Settings -->
	<section>
		<GlobalSettings />
	</section>
</div>

<!-- Add MCP Modal -->
{#if showAddMcp}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add New MCP</h2>
				<McpForm onSubmit={handleCreateMcp} onCancel={() => (showAddMcp = false)} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit MCP Modal -->
{#if editingMcp}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit MCP</h2>
				<McpForm
					initialValues={editingMcp}
					onSubmit={handleUpdateMcp}
					onCancel={() => (editingMcp = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<!-- Delete MCP Confirmation -->
<ConfirmDialog
	open={!!deletingMcp}
	title="Delete MCP"
	message="Are you sure you want to delete '{deletingMcp?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteMcp}
	onCancel={() => (deletingMcp = null)}
/>

<!-- Remove Project Confirmation -->
<ConfirmDialog
	open={!!deletingProject}
	title="Remove Project"
	message="Are you sure you want to remove '{deletingProject?.name}'? This won't delete any files."
	confirmText="Remove"
	variant="warning"
	onConfirm={handleRemoveProject}
	onCancel={() => (deletingProject = null)}
/>
