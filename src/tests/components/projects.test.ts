import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte/pure';

vi.mock('$lib/stores', () => ({
	projectsStore: {
		projects: [],
		sortedProjects: [],
		isLoading: false,
		loadProjects: vi.fn(),
		syncProjectConfig: vi.fn(),
		getProjectById: vi.fn(),
		toggleFavorite: vi.fn(),
		assignMcpToProject: vi.fn(),
		removeMcpFromProject: vi.fn(),
		toggleProjectMcp: vi.fn(),
		globalMcps: [],
		syncGlobalConfig: vi.fn(),
		loadGlobalMcps: vi.fn(),
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
		getProjectSkills: vi.fn().mockResolvedValue([]),
		getSkillById: vi.fn(),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectSkill: vi.fn(),
		globalSkills: [],
		loadGlobalSkills: vi.fn()
	},
	subagentLibrary: {
		subagents: [],
		getProjectSubAgents: vi.fn().mockResolvedValue([]),
		getSubAgentById: vi.fn(),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectSubAgent: vi.fn(),
		globalSubAgents: [],
		loadGlobalSubAgents: vi.fn()
	},
	commandLibrary: {
		commands: [],
		getProjectCommands: vi.fn().mockResolvedValue([]),
		getCommandById: vi.fn(),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectCommand: vi.fn(),
		globalCommands: [],
		loadGlobalCommands: vi.fn()
	},
	hookLibrary: {
		hooks: [],
		getProjectHooks: vi.fn().mockResolvedValue([]),
		getHookById: vi.fn(),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectHook: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	claudeSettingsLibrary: {
		isLoading: false,
		error: null,
		selectedScope: 'project',
		currentScopeSettings: null,
		load: vi.fn(),
		save: vi.fn(),
		setScope: vi.fn()
	}
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$app/stores', () => ({
	page: { subscribe: vi.fn((cb: any) => { cb({ url: new URL('http://localhost/projects/1?tab=tools') }); return () => {}; }) }
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
	open: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('$lib/components/settings', () => ({
	SETTINGS_CATEGORIES: []
}));

vi.mock('$lib/components/claude-settings', () => ({
	ModelConfigEditor: {},
	AttributionEditor: {}
}));

vi.mock('$lib/components/sandbox', () => ({
	SandboxConfigEditor: {}
}));

vi.mock('$lib/components/plugins', () => ({
	PluginListEditor: {},
	MarketplaceEditor: {}
}));

vi.mock('$lib/components/env-vars', () => ({
	EnvVarsEditor: {}
}));

vi.mock('$lib/components/ui-toggles', () => ({
	UITogglesEditor: {}
}));

vi.mock('$lib/components/file-suggestion', () => ({
	FileSuggestionEditor: {}
}));

vi.mock('$lib/components/session-cleanup', () => ({
	SessionCleanupEditor: {}
}));

vi.mock('$lib/components/auth-helpers', () => ({
	AuthHelpersEditor: {}
}));

vi.mock('$lib/components/mcp-approval', () => ({
	McpApprovalEditor: {}
}));

describe('ProjectCard Component', () => {
	let ProjectCard: any;

	const mockProject = {
		id: 1,
		name: 'Test Project',
		path: '/home/user/project',
		editorType: 'claude_code' as const,
		assignedMcps: [
			{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'MCP1', type: 'stdio' as const } }
		],
		hasMcpFile: false,
		hasSettingsFile: false,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectCard.svelte');
		ProjectCard = mod.default;
	});

	it('should render project name', () => {
		render(ProjectCard, { props: { project: mockProject } });
		expect(screen.getByText('Test Project')).toBeInTheDocument();
	});

	it('should render project path', () => {
		render(ProjectCard, { props: { project: mockProject } });
		expect(screen.getByText('/home/user/project')).toBeInTheDocument();
	});

	it('should show Claude badge for claude_code editor type', () => {
		render(ProjectCard, { props: { project: mockProject } });
		expect(screen.getByText('Claude')).toBeInTheDocument();
	});

	it('should show OpenCode badge for opencode editor type', () => {
		render(ProjectCard, {
			props: { project: { ...mockProject, editorType: 'opencode' } }
		});
		expect(screen.getByText('OpenCode')).toBeInTheDocument();
	});

	it('should show .mcp.json badge when hasMcpFile is true', () => {
		render(ProjectCard, {
			props: { project: { ...mockProject, hasMcpFile: true } }
		});
		expect(screen.getByText('.mcp.json')).toBeInTheDocument();
	});

	it('should not show .mcp.json badge when hasMcpFile is false', () => {
		render(ProjectCard, {
			props: { project: { ...mockProject, hasMcpFile: false } }
		});
		expect(screen.queryByText('.mcp.json')).not.toBeInTheDocument();
	});

	it('should show MCP count when there are MCPs', () => {
		render(ProjectCard, { props: { project: mockProject } });
		expect(screen.getByText('1/1')).toBeInTheDocument();
	});

	it('should show 0 for MCP count when no MCPs', () => {
		render(ProjectCard, {
			props: { project: { ...mockProject, assignedMcps: [] } }
		});
		expect(screen.getAllByText('0').length).toBeGreaterThan(0);
	});

	it('should show skills count as 0 when no preloaded skills', () => {
		render(ProjectCard, { props: { project: mockProject } });
		// Skills, agents both show 0 initially
		const zeros = screen.getAllByText('0');
		expect(zeros.length).toBeGreaterThanOrEqual(2);
	});

	it('should show skills count when preloadedSkills provided', () => {
		const skills = [
			{ id: 1, skillId: 1, isEnabled: true, skill: { id: 1, name: 'Skill1' } },
			{ id: 2, skillId: 2, isEnabled: false, skill: { id: 2, name: 'Skill2' } }
		];
		render(ProjectCard, {
			props: { project: mockProject, preloadedSkills: skills }
		});
		expect(screen.getByText('1/2')).toBeInTheDocument();
	});

	it('should show agents count when preloadedAgents provided', () => {
		const agents = [
			{ id: 1, subagentId: 1, isEnabled: true, subagent: { id: 1, name: 'Agent1' } }
		];
		render(ProjectCard, {
			props: { project: mockProject, preloadedAgents: agents }
		});
		// enabled/total = 1/1 - but this conflicts with MCP count, so just check aria label
		const el = screen.getByLabelText('1 of 1 agents enabled');
		expect(el).toBeInTheDocument();
	});

	it('should render FavoriteButton when onFavoriteToggle provided', () => {
		render(ProjectCard, {
			props: { project: mockProject, onFavoriteToggle: vi.fn() }
		});
		// FavoriteButton renders a button with accessible name
		const favBtn = screen.getByLabelText(`Add ${mockProject.name} to favorites`);
		expect(favBtn).toBeInTheDocument();
	});

	it('should not render FavoriteButton when onFavoriteToggle not provided', () => {
		render(ProjectCard, {
			props: { project: mockProject }
		});
		expect(screen.queryByLabelText(`Add ${mockProject.name} to favorites`)).not.toBeInTheDocument();
	});

	it('should call onClick when card is clicked', async () => {
		const onClick = vi.fn();
		render(ProjectCard, {
			props: { project: mockProject, onClick }
		});
		// The card is a div with role="button" and tabindex="0"
		const cards = screen.getAllByRole('button');
		const card = cards.find(el => el.getAttribute('tabindex') === '0');
		expect(card).toBeTruthy();
		await fireEvent.click(card!);
		expect(onClick).toHaveBeenCalled();
	});

	it('should call onClick on Enter key press', async () => {
		const onClick = vi.fn();
		render(ProjectCard, {
			props: { project: mockProject, onClick }
		});
		const cards = screen.getAllByRole('button');
		const card = cards.find(el => el.getAttribute('tabindex') === '0');
		expect(card).toBeTruthy();
		await fireEvent.keyDown(card!, { key: 'Enter' });
		expect(onClick).toHaveBeenCalled();
	});

	it('should call onClick on Space key press', async () => {
		const onClick = vi.fn();
		render(ProjectCard, {
			props: { project: mockProject, onClick }
		});
		const cards = screen.getAllByRole('button');
		const card = cards.find(el => el.getAttribute('tabindex') === '0');
		expect(card).toBeTruthy();
		await fireEvent.keyDown(card!, { key: ' ' });
		expect(onClick).toHaveBeenCalled();
	});

	it('should show enabled/total counts with mixed enabled MCPs', () => {
		const project = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'MCP1', type: 'stdio' as const } },
				{ id: 2, mcpId: 2, isEnabled: false, mcp: { id: 2, name: 'MCP2', type: 'sse' as const } },
				{ id: 3, mcpId: 3, isEnabled: true, mcp: { id: 3, name: 'MCP3', type: 'http' as const } }
			]
		};
		render(ProjectCard, { props: { project } });
		expect(screen.getByLabelText('2 of 3 MCPs enabled')).toBeInTheDocument();
	});
});

describe('ProjectList Component', () => {
	let ProjectList: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectList.svelte');
		ProjectList = mod.default;
	});

	it('should show empty state when no projects', () => {
		render(ProjectList, { props: {} });
		expect(screen.getByText('No projects added')).toBeInTheDocument();
	});

	it('should show Add Project button when callback provided', () => {
		render(ProjectList, {
			props: { onAddProject: vi.fn() }
		});
		expect(screen.getByText('Add Project')).toBeInTheDocument();
	});

	it('should show "Add Your First Project" button in empty state with callback', () => {
		render(ProjectList, {
			props: { onAddProject: vi.fn() }
		});
		expect(screen.getByText('Add Your First Project')).toBeInTheDocument();
	});

	it('should not show Add Project button when callback not provided', () => {
		render(ProjectList, { props: {} });
		expect(screen.queryByText('Add Project')).not.toBeInTheDocument();
	});

	it('should show description text', () => {
		render(ProjectList, { props: {} });
		expect(screen.getByText('Click a project to open its dashboard')).toBeInTheDocument();
	});

	it('should show Projects header', () => {
		render(ProjectList, { props: {} });
		expect(screen.getByText('Projects')).toBeInTheDocument();
	});
});

describe('ProjectToolsPanel Component', () => {
	let ProjectToolsPanel: any;

	const mockProject = {
		id: 1,
		name: 'Test',
		path: '/test',
		editorType: 'claude_code' as const,
		assignedMcps: [],
		hasMcpFile: false,
		hasSettingsFile: false,
		isFavorite: false,
		createdAt: '',
		updatedAt: ''
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectToolsPanel.svelte');
		ProjectToolsPanel = mod.default;
	});

	it('should render all tabs', () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getAllByText(/MCPs/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Skills/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Agents/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Commands/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Hooks/).length).toBeGreaterThan(0);
	});

	it('should show empty MCP state on default tab', () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getByText('No MCPs assigned yet')).toBeInTheDocument();
	});

	it('should show "Add MCPs from the library below" hint', () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getByText('Add MCPs from the library below')).toBeInTheDocument();
	});

	it('should show assigned MCP count of 0', () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getByText('Assigned MCPs (0)')).toBeInTheDocument();
	});

	it('should show available MCPs count of 0', () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getByText('Available MCPs (0)')).toBeInTheDocument();
	});

	it('should show "All MCPs are assigned" when no available MCPs and some assigned', () => {
		// mcpLibrary.mcps is [] and project has no MCPs, so available is 0
		// Since mcps store is empty and assignedMcps is empty, available is 0
		render(ProjectToolsPanel, { props: { project: mockProject } });
		expect(screen.getByText('All MCPs are assigned')).toBeInTheDocument();
	});

	it('should show assigned MCPs when project has MCPs', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'TestMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectToolsPanel, { props: { project: projectWithMcps } });
		expect(screen.getByText('TestMCP')).toBeInTheDocument();
		expect(screen.getByText('(stdio)')).toBeInTheDocument();
	});

	it('should show disabled styling for disabled MCP assignment', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: false, mcp: { id: 1, name: 'DisabledMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectToolsPanel, { props: { project: projectWithMcps } });
		const mcpEl = screen.getByText('DisabledMCP');
		expect(mcpEl.className).toContain('line-through');
	});

	it('should show toggle switch for assigned MCPs', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'TestMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectToolsPanel, { props: { project: projectWithMcps } });
		const toggle = screen.getByRole('switch');
		expect(toggle).toBeInTheDocument();
		expect(toggle.getAttribute('aria-checked')).toBe('true');
	});

	it('should show remove button for assigned MCPs', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'TestMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectToolsPanel, { props: { project: projectWithMcps } });
		expect(screen.getByTitle('Remove from project')).toBeInTheDocument();
	});

	it('should switch to Skills tab when clicked', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const skillsTab = screen.getAllByText(/Skills/)[0];
		await fireEvent.click(skillsTab);
		expect(screen.getByText('No skills assigned yet')).toBeInTheDocument();
	});

	it('should switch to Agents tab when clicked', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const agentsTab = screen.getAllByText(/Agents/)[0];
		await fireEvent.click(agentsTab);
		expect(screen.getByText('No agents assigned yet')).toBeInTheDocument();
	});

	it('should switch to Commands tab when clicked', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const commandsTab = screen.getAllByText(/Commands/)[0];
		await fireEvent.click(commandsTab);
		expect(screen.getByText('No commands assigned yet')).toBeInTheDocument();
	});

	it('should switch to Hooks tab when clicked', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const hooksTab = screen.getAllByText(/Hooks/)[0];
		await fireEvent.click(hooksTab);
		expect(screen.getByText('No hooks assigned yet')).toBeInTheDocument();
	});

	it('should show "All skills are assigned" when no available skills', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const skillsTab = screen.getAllByText(/Skills/)[0];
		await fireEvent.click(skillsTab);
		expect(screen.getByText('All skills are assigned')).toBeInTheDocument();
	});

	it('should show "All agents are assigned" when no available agents', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const agentsTab = screen.getAllByText(/Agents/)[0];
		await fireEvent.click(agentsTab);
		expect(screen.getByText('All agents are assigned')).toBeInTheDocument();
	});

	it('should show "All commands are assigned" when no available commands', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const commandsTab = screen.getAllByText(/Commands/)[0];
		await fireEvent.click(commandsTab);
		expect(screen.getByText('All commands are assigned')).toBeInTheDocument();
	});

	it('should show "All hooks are assigned" when no available hooks', async () => {
		render(ProjectToolsPanel, { props: { project: mockProject } });
		const hooksTab = screen.getAllByText(/Hooks/)[0];
		await fireEvent.click(hooksTab);
		expect(screen.getByText('All hooks are assigned')).toBeInTheDocument();
	});
});

describe('ProjectDashboard Component', () => {
	let ProjectDashboard: any;

	const mockProject = {
		id: 1,
		name: 'Dashboard Project',
		path: '/home/user/dashboard',
		editorType: 'claude_code' as const,
		assignedMcps: [
			{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'MCP1', type: 'stdio' as const } }
		],
		hasMcpFile: false,
		hasSettingsFile: false,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectDashboard.svelte');
		ProjectDashboard = mod.default;
	});

	it('should render project name', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByText('Dashboard Project')).toBeInTheDocument();
	});

	it('should render project path', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByText('/home/user/dashboard')).toBeInTheDocument();
	});

	it('should show Claude Code editor type display', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByText('Claude Code')).toBeInTheDocument();
	});

	it('should show OpenCode editor type display for opencode projects', () => {
		render(ProjectDashboard, {
			props: { project: { ...mockProject, editorType: 'opencode' } }
		});
		// The dropdown button shows OpenCode
		expect(screen.getAllByText('OpenCode').length).toBeGreaterThan(0);
	});

	it('should show Tools and Settings tabs', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByText('Tools')).toBeInTheDocument();
		expect(screen.getByText('Settings')).toBeInTheDocument();
	});

	it('should show tools count badge when project has MCPs', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		// The badge shows the count (1 for 1 MCP)
		const badges = screen.getAllByText('1');
		expect(badges.length).toBeGreaterThan(0);
	});

	it('should not show tools count badge when project has no MCPs', () => {
		render(ProjectDashboard, {
			props: { project: { ...mockProject, assignedMcps: [] } }
		});
		// Should not have a count badge
		expect(screen.queryByText(/^\d+$/)).toBeNull();
	});

	it('should show Back to Projects button', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByTitle('Back to Projects')).toBeInTheDocument();
	});

	it('should show Open Folder button', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByTitle('Open project folder')).toBeInTheDocument();
	});

	it('should show Sync Config button', () => {
		render(ProjectDashboard, { props: { project: mockProject } });
		expect(screen.getByTitle('Sync config files')).toBeInTheDocument();
	});
});

describe('ProjectSettingsPanel Component', () => {
	let ProjectSettingsPanel: any;

	const mockProject = {
		id: 1,
		name: 'Test',
		path: '/test',
		editorType: 'claude_code' as const,
		assignedMcps: [],
		hasMcpFile: false,
		hasSettingsFile: false,
		isFavorite: false,
		createdAt: '',
		updatedAt: ''
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectSettingsPanel.svelte');
		ProjectSettingsPanel = mod.default;
	});

	it('should render without crashing', () => {
		render(ProjectSettingsPanel, { props: { project: mockProject } });
		expect(document.body).toBeTruthy();
	});

	it('should show no settings available when currentScopeSettings is null', () => {
		render(ProjectSettingsPanel, { props: { project: mockProject } });
		expect(screen.getByText('No settings available for this scope')).toBeInTheDocument();
	});

	it('should show refresh button', () => {
		render(ProjectSettingsPanel, { props: { project: mockProject } });
		expect(screen.getByTitle('Refresh from settings files')).toBeInTheDocument();
	});
});

describe('ProjectDetail Component', () => {
	let ProjectDetail: any;

	const mockProject = {
		id: 1,
		name: 'Detail Project',
		path: '/home/user/detail',
		editorType: 'claude_code' as const,
		assignedMcps: [],
		hasMcpFile: false,
		hasSettingsFile: false,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/projects/ProjectDetail.svelte');
		ProjectDetail = mod.default;
	});

	it('should render project name in dialog', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByText('Detail Project')).toBeInTheDocument();
	});

	it('should render project path in dialog', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByText('/home/user/detail')).toBeInTheDocument();
	});

	it('should show MCPs, Skills, Agents, and Commands tabs', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getAllByText(/MCPs/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Skills/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Agents/).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/Commands/).length).toBeGreaterThan(0);
	});

	it('should show Claude Code editor type', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByText('Claude Code')).toBeInTheDocument();
	});

	it('should show OpenCode editor type for opencode projects', () => {
		render(ProjectDetail, {
			props: { project: { ...mockProject, editorType: 'opencode' }, onClose: vi.fn() }
		});
		expect(screen.getAllByText('OpenCode').length).toBeGreaterThan(0);
	});

	it('should show empty MCPs state', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByText('No MCPs assigned yet')).toBeInTheDocument();
	});

	it('should show "All MCPs are assigned" when no available MCPs', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByText('All MCPs are assigned')).toBeInTheDocument();
	});

	it('should show assigned MCPs with type', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'TestStdio', type: 'stdio' as const } }
			]
		};
		render(ProjectDetail, { props: { project: projectWithMcps, onClose: vi.fn() } });
		expect(screen.getByText('TestStdio')).toBeInTheDocument();
		expect(screen.getByText('(stdio)')).toBeInTheDocument();
	});

	it('should show toggle switch for MCP with correct aria-checked', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: false, mcp: { id: 1, name: 'TestMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectDetail, { props: { project: projectWithMcps, onClose: vi.fn() } });
		const toggle = screen.getByRole('switch');
		expect(toggle.getAttribute('aria-checked')).toBe('false');
	});

	it('should show Skills tab with empty state when clicked', async () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		const skillsTab = screen.getAllByText(/Skills/)[0];
		await fireEvent.click(skillsTab);
		expect(screen.getByText('No skills assigned yet')).toBeInTheDocument();
	});

	it('should show Agents tab with empty state when clicked', async () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		const agentsTab = screen.getAllByText(/Agents/)[0];
		await fireEvent.click(agentsTab);
		expect(screen.getByText('No agents assigned yet')).toBeInTheDocument();
	});

	it('should show Commands tab with empty state when clicked', async () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		const commandsTab = screen.getAllByText(/Commands/)[0];
		await fireEvent.click(commandsTab);
		expect(screen.getByText('No commands assigned yet')).toBeInTheDocument();
	});

	it('should have close button', () => {
		const onClose = vi.fn();
		render(ProjectDetail, { props: { project: mockProject, onClose } });
		// The X close button exists
		const closeButtons = document.querySelectorAll('button');
		expect(closeButtons.length).toBeGreaterThan(0);
	});

	it('should have modal dialog role', () => {
		render(ProjectDetail, { props: { project: mockProject, onClose: vi.fn() } });
		expect(screen.getByRole('dialog')).toBeInTheDocument();
	});

	it('should show line-through style for disabled MCPs', () => {
		const projectWithMcps = {
			...mockProject,
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: false, mcp: { id: 1, name: 'DisabledMCP', type: 'stdio' as const } }
			]
		};
		render(ProjectDetail, { props: { project: projectWithMcps, onClose: vi.fn() } });
		const mcpEl = screen.getByText('DisabledMCP');
		expect(mcpEl.className).toContain('line-through');
	});
});

describe('Projects index.ts exports', () => {
	let projectExports: any;

	beforeAll(async () => {
		projectExports = await import('$lib/components/projects');
	});

	it('should export all project components', () => {
		expect(projectExports.ProjectCard).toBeDefined();
		expect(projectExports.ProjectDashboard).toBeDefined();
		expect(projectExports.ProjectDetail).toBeDefined();
		expect(projectExports.ProjectList).toBeDefined();
		expect(projectExports.ProjectSettingsPanel).toBeDefined();
		expect(projectExports.ProjectToolsPanel).toBeDefined();
	});
});
