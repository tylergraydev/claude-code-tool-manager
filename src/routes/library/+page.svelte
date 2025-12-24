<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Header } from '$lib/components/layout';
	import { McpLibrary, McpForm, McpTestModal } from '$lib/components/mcp';
	import { ConfirmDialog } from '$lib/components/shared';
	import { mcpLibrary, notifications } from '$lib/stores';
	import type { Mcp, GatewayMcp } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddMcp = $state(false);
	let editingMcp = $state<Mcp | null>(null);
	let deletingMcp = $state<Mcp | null>(null);
	let testingMcp = $state<Mcp | null>(null);
	let gatewayMcpIds = $state<Set<number>>(new Set());

	onMount(async () => {
		await loadGatewayMcps();
	});

	async function loadGatewayMcps() {
		try {
			const gatewayMcps = await invoke<GatewayMcp[]>('get_gateway_mcps');
			gatewayMcpIds = new Set(gatewayMcps.map(gm => gm.mcpId));
		} catch (err) {
			console.error('Failed to load gateway MCPs:', err);
		}
	}

	async function handleGatewayToggle(mcp: Mcp, enabled: boolean) {
		try {
			if (enabled) {
				await invoke('add_mcp_to_gateway', { mcpId: mcp.id });
				gatewayMcpIds = new Set([...gatewayMcpIds, mcp.id]);
				notifications.success(`Added "${mcp.name}" to Gateway`);
			} else {
				await invoke('remove_mcp_from_gateway', { mcpId: mcp.id });
				const newIds = new Set(gatewayMcpIds);
				newIds.delete(mcp.id);
				gatewayMcpIds = newIds;
				notifications.success(`Removed "${mcp.name}" from Gateway`);
			}
		} catch (err) {
			notifications.error(enabled ? 'Failed to add to Gateway' : 'Failed to remove from Gateway');
			console.error('Gateway toggle error:', err);
		}
	}

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
</script>

<Header
	title="MCP Library"
	subtitle="All your MCP servers in one place - drag them to projects or global settings"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddMcp = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Add MCP
		</button>
	</div>

	<McpLibrary
		onEdit={(mcp) => (editingMcp = mcp)}
		onDelete={(mcp) => (deletingMcp = mcp)}
		onDuplicate={handleDuplicateMcp}
		onTest={(mcp) => (testingMcp = mcp)}
		showGatewayToggle={true}
		{gatewayMcpIds}
		onGatewayToggle={handleGatewayToggle}
	/>
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

<ConfirmDialog
	open={!!deletingMcp}
	title="Delete MCP"
	message="Are you sure you want to delete '{deletingMcp?.name}'? This will remove it from all projects."
	confirmText="Delete"
	onConfirm={handleDeleteMcp}
	onCancel={() => (deletingMcp = null)}
/>

<!-- Test MCP Modal -->
{#if testingMcp}
	<McpTestModal mcp={testingMcp} onClose={() => (testingMcp = null)} />
{/if}
