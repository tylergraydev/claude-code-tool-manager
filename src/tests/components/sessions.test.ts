import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	sessionStore: {
		projects: [],
		sessions: [],
		selectedProject: null,
		isLoading: false,
		isLoadingProjects: false,
		load: vi.fn(),
		loadProjects: vi.fn()
	}
}));

vi.mock('$lib/types/usage', () => ({
	formatCompactNumber: vi.fn((n: number) => String(n)),
	formatDuration: vi.fn(() => '1h'),
	formatCost: vi.fn((n: number) => `$${n.toFixed(2)}`),
	formatModelName: vi.fn((m: string) => m),
	estimateSessionCost: vi.fn(() => 0.05)
}));

vi.mock('$lib/types/session', () => ({
	totalTokens: vi.fn(() => 1500),
	projectTotalTokens: vi.fn((p: any) => p.totalInputTokens + p.totalOutputTokens)
}));

const mockSession = {
	sessionId: 'session-1',
	firstTimestamp: '2024-01-01T10:00:00Z',
	lastTimestamp: '2024-01-01T11:00:00Z',
	durationMs: 3600000,
	userMessageCount: 5,
	assistantMessageCount: 5,
	totalInputTokens: 1000,
	totalOutputTokens: 500,
	totalCacheReadTokens: 200,
	totalCacheCreationTokens: 100,
	modelsUsed: ['claude-3-sonnet'],
	toolUsage: { Read: 3, Edit: 2 },
	gitBranch: 'main',
	firstUserMessage: 'Hello, help me refactor this code',
	hasToolUse: true
};

const mockProject = {
	folderName: 'test-project',
	inferredPath: '/home/user/test-project',
	sessionCount: 10,
	totalInputTokens: 5000,
	totalOutputTokens: 3000,
	totalCacheReadTokens: 1000,
	totalCacheCreationTokens: 500,
	modelsUsed: ['claude-3-sonnet'],
	toolUsage: { Read: 15, Edit: 8 },
	latestSession: '2024-06-01',
	earliestSession: '2024-01-01'
};

describe('SessionListTable Component', () => {
	let SessionListTable: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sessions/SessionListTable.svelte');
		SessionListTable = mod.default;
	});

	it('should render heading with session count', () => {
		render(SessionListTable, {
			props: {
				sessions: [],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('Sessions (0)')).toBeInTheDocument();
	});

	it('should show No sessions found when empty', () => {
		render(SessionListTable, {
			props: {
				sessions: [],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('No sessions found')).toBeInTheDocument();
	});

	it('should render column headers', () => {
		render(SessionListTable, {
			props: {
				sessions: [mockSession],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('Date')).toBeInTheDocument();
		expect(screen.getByText('Duration')).toBeInTheDocument();
		expect(screen.getByText('Messages')).toBeInTheDocument();
		expect(screen.getByText('Tokens')).toBeInTheDocument();
		expect(screen.getByText('Est. Cost')).toBeInTheDocument();
		expect(screen.getByText('Model')).toBeInTheDocument();
		expect(screen.getByText('Branch')).toBeInTheDocument();
		expect(screen.getByText('First Prompt')).toBeInTheDocument();
	});

	it('should render session count in heading', () => {
		render(SessionListTable, {
			props: {
				sessions: [mockSession],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('Sessions (1)')).toBeInTheDocument();
	});

	it('should render model badge', () => {
		render(SessionListTable, {
			props: {
				sessions: [mockSession],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('claude-3-sonnet')).toBeInTheDocument();
	});

	it('should render git branch', () => {
		render(SessionListTable, {
			props: {
				sessions: [mockSession],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('main')).toBeInTheDocument();
	});

	it('should show dash when no git branch', () => {
		const noGitSession = { ...mockSession, gitBranch: null };
		render(SessionListTable, {
			props: {
				sessions: [noGitSession],
				selectedSessionId: null,
				sortField: 'date' as any,
				sortDirection: 'desc',
				onSelectSession: vi.fn(),
				onSort: vi.fn()
			}
		});
		expect(screen.getByText('-')).toBeInTheDocument();
	});
});

describe('ProjectSelector Component', () => {
	let ProjectSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sessions/ProjectSelector.svelte');
		ProjectSelector = mod.default;
	});

	it('should render Projects heading', () => {
		render(ProjectSelector, {
			props: { projects: [], selectedFolder: null, onSelect: vi.fn() }
		});
		expect(screen.getByText('Projects')).toBeInTheDocument();
	});

	it('should render project items', () => {
		render(ProjectSelector, {
			props: { projects: [mockProject], selectedFolder: null, onSelect: vi.fn() }
		});
		expect(screen.getByText('user/test-project')).toBeInTheDocument();
		expect(screen.getByText('10 sessions')).toBeInTheDocument();
	});

	it('should call onSelect when project clicked', async () => {
		const onSelect = vi.fn();
		render(ProjectSelector, {
			props: { projects: [mockProject], selectedFolder: null, onSelect }
		});
		await fireEvent.click(screen.getByText('user/test-project'));
		expect(onSelect).toHaveBeenCalledWith('test-project');
	});
});

describe('ProjectOverviewCards Component', () => {
	let ProjectOverviewCards: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sessions/ProjectOverviewCards.svelte');
		ProjectOverviewCards = mod.default;
	});

	it('should render overview cards', () => {
		render(ProjectOverviewCards, { props: { projects: [mockProject] } });
		expect(screen.getByText('Projects')).toBeInTheDocument();
		expect(screen.getByText('Total Sessions')).toBeInTheDocument();
		expect(screen.getByText('Total Tokens')).toBeInTheDocument();
		expect(screen.getByText('Models Used')).toBeInTheDocument();
		expect(screen.getByText('Est. API Cost')).toBeInTheDocument();
	});

	it('should show project count', () => {
		render(ProjectOverviewCards, { props: { projects: [mockProject] } });
		expect(screen.getAllByText('1').length).toBeGreaterThan(0);
	});

	it('should show cost subtitle', () => {
		render(ProjectOverviewCards, { props: { projects: [mockProject] } });
		expect(screen.getByText('if billed at API rates')).toBeInTheDocument();
	});
});

describe('ToolUsageChart Component', () => {
	let ToolUsageChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sessions/ToolUsageChart.svelte');
		ToolUsageChart = mod.default;
	});

	it('should render Tool Usage heading', () => {
		render(ToolUsageChart, { props: { toolUsage: {} } });
		expect(screen.getByText('Tool Usage')).toBeInTheDocument();
	});

	it('should show empty state when no usage data', () => {
		render(ToolUsageChart, { props: { toolUsage: {} } });
		expect(screen.getByText('No tool usage data')).toBeInTheDocument();
	});

	it('should render SVG chart when data present', () => {
		const { container } = render(ToolUsageChart, {
			props: { toolUsage: { Read: 10, Edit: 5 } }
		});
		expect(container.querySelector('svg')).toBeInTheDocument();
	});
});

describe('SessionDetailPanel Component', () => {
	let SessionDetailPanel: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sessions/SessionDetailPanel.svelte');
		SessionDetailPanel = mod.default;
	});

	const mockDetail = {
		messages: [
			{
				role: 'user',
				contentPreview: 'Help me fix a bug',
				timestamp: '2024-01-01T10:00:00Z',
				model: null,
				toolCalls: [],
				usage: null
			},
			{
				role: 'assistant',
				contentPreview: 'I can help with that.',
				timestamp: '2024-01-01T10:00:05Z',
				model: 'claude-3-sonnet',
				toolCalls: [{ toolName: 'Read' }],
				usage: { inputTokens: 500, outputTokens: 200, cacheReadInputTokens: 100 }
			}
		]
	};

	it('should render Session Transcript heading', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('Session Transcript')).toBeInTheDocument();
	});

	it('should show message count', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('2 messages')).toBeInTheDocument();
	});

	it('should render user message', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('Help me fix a bug')).toBeInTheDocument();
	});

	it('should render assistant message', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('I can help with that.')).toBeInTheDocument();
	});

	it('should show role badges', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('User')).toBeInTheDocument();
		expect(screen.getByText('Assistant')).toBeInTheDocument();
	});

	it('should render tool calls', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('Read')).toBeInTheDocument();
	});

	it('should render model badge on assistant message', () => {
		render(SessionDetailPanel, { props: { detail: mockDetail, onClose: vi.fn() } });
		expect(screen.getByText('claude-3-sonnet')).toBeInTheDocument();
	});
});
