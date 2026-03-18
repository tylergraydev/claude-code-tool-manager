import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	projectsStore: {
		globalMcps: [],
		syncGlobalConfig: vi.fn(),
		addGlobalMcp: vi.fn(),
		removeGlobalMcp: vi.fn(),
		toggleGlobalMcp: vi.fn()
	},
	mcpLibrary: {
		mcps: [],
		getMcpById: vi.fn()
	},
	skillLibrary: {
		skills: [],
		getSkillById: vi.fn(),
		globalSkills: [],
		loadGlobalSkills: vi.fn(),
		addGlobalSkill: vi.fn(),
		removeGlobalSkill: vi.fn(),
		toggleGlobalSkill: vi.fn()
	},
	subagentLibrary: {
		subagents: [],
		getSubAgentById: vi.fn(),
		globalSubAgents: [],
		loadGlobalSubAgents: vi.fn(),
		addGlobalSubAgent: vi.fn(),
		removeGlobalSubAgent: vi.fn(),
		toggleGlobalSubAgent: vi.fn()
	},
	commandLibrary: {
		commands: [],
		getCommandById: vi.fn(),
		globalCommands: [],
		loadGlobalCommands: vi.fn(),
		addGlobalCommand: vi.fn(),
		removeGlobalCommand: vi.fn(),
		toggleGlobalCommand: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	debugStore: {
		isEnabled: false,
		isLoading: false,
		logFilePath: null,
		load: vi.fn(),
		enable: vi.fn(),
		disable: vi.fn(),
		openLogsFolder: vi.fn()
	}
}));

vi.mock('$lib/utils/debugLogger', () => ({
	installDebugInterceptor: vi.fn(),
	uninstallDebugInterceptor: vi.fn()
}));

describe('GlobalSettings Component', () => {
	let GlobalSettings: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/global/GlobalSettings.svelte');
		GlobalSettings = mod.default;
	});

	it('should render Global Settings heading', () => {
		render(GlobalSettings);
		expect(screen.getByText('Global Settings')).toBeInTheDocument();
	});

	it('should show subtitle', () => {
		render(GlobalSettings);
		expect(screen.getByText('Available in all projects')).toBeInTheDocument();
	});

	it('should show tabs for MCPs, Commands, Skills, and Agents', () => {
		render(GlobalSettings);
		expect(screen.getAllByText(/MCPs/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Commands/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Skills/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Agents/).length).toBeGreaterThan(0);
	});

	it('should show empty MCPs state by default', () => {
		render(GlobalSettings);
		expect(screen.getByText('No global MCPs')).toBeInTheDocument();
	});

	it('should show Debug Mode section', () => {
		render(GlobalSettings);
		expect(screen.getAllByText('Debug Mode').length).toBeGreaterThan(0);
	});

	it('should show debug description text', () => {
		render(GlobalSettings);
		expect(screen.getByText('Enable logging to help troubleshoot issues')).toBeInTheDocument();
	});

	it('should show Add MCP button when on MCPs tab', () => {
		render(GlobalSettings);
		expect(screen.getAllByText('Add MCP').length).toBeGreaterThan(0);
	});

	it('should show Sync button', () => {
		render(GlobalSettings);
		expect(screen.getByText('Sync')).toBeInTheDocument();
	});

	it('should show empty state text for MCPs', () => {
		render(GlobalSettings);
		expect(screen.getByText('Add MCPs to make them available in all projects')).toBeInTheDocument();
	});

	it('should have debug toggle switch', () => {
		render(GlobalSettings);
		const switches = screen.getAllByRole('switch');
		expect(switches.length).toBeGreaterThan(0);
	});

	it('should show tab counts as 0', () => {
		render(GlobalSettings);
		expect(screen.getByText('MCPs (0)')).toBeInTheDocument();
		expect(screen.getByText('Commands (0)')).toBeInTheDocument();
		expect(screen.getByText('Skills (0)')).toBeInTheDocument();
		expect(screen.getByText('Agents (0)')).toBeInTheDocument();
	});
});

describe('Global index.ts exports', () => {
	let globalExports: any;

	beforeAll(async () => {
		globalExports = await import('$lib/components/global');
	});

	it('should export GlobalSettings', () => {
		expect(globalExports.GlobalSettings).toBeDefined();
	});
});
