import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	subagentLibrary: {
		subagents: [],
		filteredSubAgents: [],
		isLoading: false,
		searchQuery: '',
		load: vi.fn(),
		create: vi.fn(),
		delete: vi.fn(),
		getSubAgentById: vi.fn(),
		getProjectSubAgents: vi.fn().mockResolvedValue([]),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectSubAgent: vi.fn(),
		globalSubAgents: [],
		loadGlobalSubAgents: vi.fn(),
		addGlobalSubAgent: vi.fn(),
		removeGlobalSubAgent: vi.fn(),
		toggleGlobalSubAgent: vi.fn(),
		updateSubAgent: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('$lib/utils/markdownParser', () => ({
	parseSubAgentMarkdown: vi.fn().mockReturnValue({ success: false })
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn().mockResolvedValue(null)
}));

describe('SubAgentCard Component', () => {
	let SubAgentCard: any;

	const mockSubAgent = {
		id: 1,
		name: 'Test Agent',
		description: 'A test agent',
		content: 'Do something',
		model: 'sonnet',
		tools: ['Read', 'Edit'],
		tags: ['code', 'review', 'testing'],
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/subagents/SubAgentCard.svelte');
		SubAgentCard = mod.default;
	});

	it('should render agent name', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.getByText('Test Agent')).toBeInTheDocument();
	});

	it('should render description when present', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.getByText('A test agent')).toBeInTheDocument();
	});

	it('should not render description when absent', () => {
		render(SubAgentCard, { props: { subagent: { ...mockSubAgent, description: '' } } });
		expect(screen.queryByText('A test agent')).not.toBeInTheDocument();
	});

	it('should show auto-detected badge when source is auto-detected', () => {
		render(SubAgentCard, { props: { subagent: { ...mockSubAgent, source: 'auto-detected' } } });
		expect(screen.getByText('Auto')).toBeInTheDocument();
	});

	it('should not show auto badge for user source', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.queryByText('Auto')).not.toBeInTheDocument();
	});

	it('should display model badge when model is set', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.getByText('sonnet')).toBeInTheDocument();
	});

	it('should not display model badge when no model', () => {
		render(SubAgentCard, { props: { subagent: { ...mockSubAgent, model: null } } });
		expect(screen.queryByText('sonnet')).not.toBeInTheDocument();
	});

	it('should display tool count badge', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.getByText('2 tools')).toBeInTheDocument();
	});

	it('should display singular tool text for one tool', () => {
		render(SubAgentCard, { props: { subagent: { ...mockSubAgent, tools: ['Read'] } } });
		expect(screen.getByText('1 tool')).toBeInTheDocument();
	});

	it('should not show tool badge when no tools', () => {
		render(SubAgentCard, { props: { subagent: { ...mockSubAgent, tools: [] } } });
		expect(screen.queryByText(/tool/)).not.toBeInTheDocument();
	});

	it('should show first two tags and overflow count', () => {
		render(SubAgentCard, { props: { subagent: mockSubAgent } });
		expect(screen.getByText('code')).toBeInTheDocument();
		expect(screen.getByText('review')).toBeInTheDocument();
		expect(screen.getByText('+1')).toBeInTheDocument();
	});

	it('should hide actions when showActions is false', () => {
		const { container } = render(SubAgentCard, {
			props: { subagent: mockSubAgent, showActions: false }
		});
		const buttons = container.querySelectorAll('button');
		expect(buttons.length).toBe(0);
	});

	it('should show favorite button when onFavoriteToggle provided', () => {
		render(SubAgentCard, {
			props: { subagent: mockSubAgent, onFavoriteToggle: vi.fn() }
		});
		expect(screen.getByTitle('Add to favorites')).toBeInTheDocument();
	});

	it('should show Remove from favorites when isFavorite', () => {
		render(SubAgentCard, {
			props: { subagent: { ...mockSubAgent, isFavorite: true }, onFavoriteToggle: vi.fn() }
		});
		expect(screen.getByTitle('Remove from favorites')).toBeInTheDocument();
	});

	it('should call onFavoriteToggle with correct args', async () => {
		const onFavoriteToggle = vi.fn();
		render(SubAgentCard, {
			props: { subagent: mockSubAgent, onFavoriteToggle }
		});
		await fireEvent.click(screen.getByTitle('Add to favorites'));
		expect(onFavoriteToggle).toHaveBeenCalledWith(mockSubAgent, true);
	});
});

describe('SubAgentForm Component', () => {
	let SubAgentForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/subagents/SubAgentForm.svelte');
		SubAgentForm = mod.default;
	});

	it('should render form with all fields', () => {
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByLabelText(/^Name/)).toBeInTheDocument();
		expect(screen.getByLabelText(/^Description/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Model/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Permission Mode/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Allowed Tools/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Auto-load Skills/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Sub-Agent Prompt/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Tags/)).toBeInTheDocument();
	});

	it('should show Create Sub-Agent button for new agent', () => {
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Create Sub-Agent')).toBeInTheDocument();
	});

	it('should show Update Sub-Agent button when editing', () => {
		render(SubAgentForm, {
			props: { initialValues: { name: 'test' }, onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Update Sub-Agent')).toBeInTheDocument();
	});

	it('should show Import from Markdown section', () => {
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Import from Markdown')).toBeInTheDocument();
	});

	it('should call onCancel when Cancel clicked', async () => {
		const onCancel = vi.fn();
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel } });
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('should render model options', () => {
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getAllByText('Default (inherit from parent)').length).toBeGreaterThanOrEqual(1);
		expect(screen.getByText('Sonnet')).toBeInTheDocument();
		expect(screen.getByText('Opus')).toBeInTheDocument();
		expect(screen.getByText('Haiku')).toBeInTheDocument();
	});

	it('should render permission mode options', () => {
		render(SubAgentForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Default (standard permission prompting)')).toBeInTheDocument();
	});
});

describe('SubAgentLibrary Component', () => {
	let SubAgentLibrary: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/subagents/SubAgentLibrary.svelte');
		SubAgentLibrary = mod.default;
	});

	it('should render empty state when no sub-agents', () => {
		render(SubAgentLibrary);
		expect(screen.getByText('No sub-agents in library')).toBeInTheDocument();
	});

	it('should render search bar', () => {
		render(SubAgentLibrary);
		expect(screen.getByPlaceholderText('Search sub-agents...')).toBeInTheDocument();
	});

	it('should show count', () => {
		render(SubAgentLibrary);
		expect(screen.getByText('0 sub-agents')).toBeInTheDocument();
	});
});

describe('SubAgents index.ts exports', () => {
	let exports: any;

	beforeAll(async () => {
		exports = await import('$lib/components/subagents');
	});

	it('should export all components', () => {
		expect(exports.SubAgentCard).toBeDefined();
		expect(exports.SubAgentLibrary).toBeDefined();
		expect(exports.SubAgentForm).toBeDefined();
	});
});
