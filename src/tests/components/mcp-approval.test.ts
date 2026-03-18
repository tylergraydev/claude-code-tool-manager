import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('McpApprovalEditor Component', () => {
	const mockSettings = {
		scope: 'project',
		availableModels: [],
		enableAllProjectMcpServers: undefined,
		enabledMcpjsonServers: ['server-a'],
		disabledMcpjsonServers: ['server-b']
	};

	it('should render heading', async () => {
		const { default: McpApprovalEditor } = await import('$lib/components/mcp-approval/McpApprovalEditor.svelte');
		render(McpApprovalEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('MCP Server Approval')).toBeInTheDocument();
	});

	it('should show enabled servers', async () => {
		const { default: McpApprovalEditor } = await import('$lib/components/mcp-approval/McpApprovalEditor.svelte');
		render(McpApprovalEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('server-a')).toBeInTheDocument();
	});

	it('should show disabled servers', async () => {
		const { default: McpApprovalEditor } = await import('$lib/components/mcp-approval/McpApprovalEditor.svelte');
		render(McpApprovalEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('server-b')).toBeInTheDocument();
	});

	it('should call onsave on save', async () => {
		const { default: McpApprovalEditor } = await import('$lib/components/mcp-approval/McpApprovalEditor.svelte');
		const onsave = vi.fn();
		render(McpApprovalEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save MCP Approval Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});

	it('should show Not set label for undefined enableAll', async () => {
		const { default: McpApprovalEditor } = await import('$lib/components/mcp-approval/McpApprovalEditor.svelte');
		render(McpApprovalEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Not set')).toBeInTheDocument();
	});
});

describe('MCP-approval index.ts exports', () => {
	it('should export McpApprovalEditor', async () => {
		const exports = await import('$lib/components/mcp-approval');
		expect(exports.McpApprovalEditor).toBeDefined();
	});
});
