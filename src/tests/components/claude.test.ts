import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	claudeJson: {
		isLoading: false,
		error: null,
		globalMcps: [],
		projects: [],
		loadAll: vi.fn(),
		toggleMcp: vi.fn(),
		removeMcpFromProject: vi.fn(),
		removeGlobalMcp: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

describe('ClaudeJsonView Component', () => {
	let ClaudeJsonView: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/claude/ClaudeJsonView.svelte');
		ClaudeJsonView = mod.default;
	});

	it('should render header', () => {
		render(ClaudeJsonView);
		expect(screen.getByText('Claude.json Config')).toBeInTheDocument();
	});

	it('should render subtitle', () => {
		render(ClaudeJsonView);
		expect(screen.getByText(/MCPs configured directly in/)).toBeInTheDocument();
	});

	it('should render Refresh button', () => {
		render(ClaudeJsonView);
		expect(screen.getByText('Refresh')).toBeInTheDocument();
	});

	it('should show empty state when no MCPs', () => {
		render(ClaudeJsonView);
		expect(screen.getByText('No MCPs in claude.json')).toBeInTheDocument();
		expect(screen.getByText('MCPs configured through Claude Code will appear here')).toBeInTheDocument();
	});

	it('should show loading state', async () => {
		const { claudeJson } = await import('$lib/stores');
		(claudeJson as any).isLoading = true;
		render(ClaudeJsonView);
		const spinner = document.querySelector('.animate-spin');
		expect(spinner).toBeInTheDocument();
		(claudeJson as any).isLoading = false;
	});

	it('should show error state', async () => {
		const { claudeJson } = await import('$lib/stores');
		(claudeJson as any).error = 'Failed to load';
		render(ClaudeJsonView);
		expect(screen.getByText('Failed to load')).toBeInTheDocument();
		(claudeJson as any).error = null;
	});

	it('should render global MCPs when present', async () => {
		const { claudeJson } = await import('$lib/stores');
		(claudeJson as any).globalMcps = [
			{ name: 'test-mcp', type: 'stdio', isEnabled: true, projectPath: null }
		];
		render(ClaudeJsonView);
		expect(screen.getByText('Global MCPs')).toBeInTheDocument();
		expect(screen.getByText('test-mcp')).toBeInTheDocument();
		expect(screen.getByText('(stdio)')).toBeInTheDocument();
		(claudeJson as any).globalMcps = [];
	});

	it('should render project MCPs with expand/collapse', async () => {
		const { claudeJson } = await import('$lib/stores');
		(claudeJson as any).projects = [
			{
				path: '/home/user/my-project',
				mcps: [
					{ name: 'project-mcp', type: 'sse', isEnabled: true, projectPath: '/home/user/my-project' }
				]
			}
		];
		render(ClaudeJsonView);
		expect(screen.getByText('Project MCPs')).toBeInTheDocument();
		expect(screen.getByText('my-project')).toBeInTheDocument();
		expect(screen.getByText('1 MCP')).toBeInTheDocument();
		(claudeJson as any).projects = [];
	});

	it('should show plural MCPs text', async () => {
		const { claudeJson } = await import('$lib/stores');
		(claudeJson as any).projects = [
			{
				path: '/home/user/my-project',
				mcps: [
					{ name: 'mcp-1', type: 'stdio', isEnabled: true, projectPath: '/home/user/my-project' },
					{ name: 'mcp-2', type: 'sse', isEnabled: true, projectPath: '/home/user/my-project' }
				]
			}
		];
		render(ClaudeJsonView);
		expect(screen.getByText('2 MCPs')).toBeInTheDocument();
		(claudeJson as any).projects = [];
	});

	it('should call loadAll on Refresh click', async () => {
		const { claudeJson } = await import('$lib/stores');
		render(ClaudeJsonView);
		await fireEvent.click(screen.getByText('Refresh'));
		expect(claudeJson.loadAll).toHaveBeenCalled();
	});
});

describe('Claude index.ts exports', () => {
	let claudeExports: any;

	beforeAll(async () => {
		claudeExports = await import('$lib/components/claude');
	});

	it('should export ClaudeJsonView', () => {
		expect(claudeExports.ClaudeJsonView).toBeDefined();
	});
});
