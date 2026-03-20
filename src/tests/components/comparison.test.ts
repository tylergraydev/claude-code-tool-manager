import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/stores/comparisonStore.svelte', () => ({
	comparisonStore: {
		selectedFolders: new Set(),
		selectedProjects: [],
		comparisonData: [],
		isLoading: false,
		load: vi.fn(),
		toggleProject: vi.fn(),
		clearSelection: vi.fn()
	},
	PROJECT_COLORS: ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6']
}));

vi.mock('$lib/stores', () => ({
	comparisonStore: {
		selectedFolders: new Set(),
		selectedProjects: [],
		comparisonData: [],
		isLoading: false,
		load: vi.fn(),
		toggleProject: vi.fn(),
		clearSelection: vi.fn()
	}
}));

vi.mock('$lib/types/usage', () => ({
	formatCompactNumber: vi.fn((n: number) => String(n)),
	formatCost: vi.fn((n: number) => `$${n.toFixed(2)}`),
	formatModelName: vi.fn((m: string) => m),
	estimateSessionCost: vi.fn(() => 0),
	getModelColor: vi.fn(() => '#888')
}));

vi.mock('$lib/types/session', () => ({
	projectTotalTokens: vi.fn((p: any) => p.totalInputTokens + p.totalOutputTokens),
	totalTokens: vi.fn(() => 0)
}));

const mockProject = {
	folderName: 'test-project',
	inferredPath: '/home/user/test-project',
	sessionCount: 10,
	totalInputTokens: 5000,
	totalOutputTokens: 3000,
	totalCacheReadTokens: 1000,
	totalCacheCreationTokens: 500,
	modelsUsed: ['claude-3-sonnet'],
	toolUsage: { Read: 5, Edit: 3 } as Record<string, number>,
	latestSession: '2024-06-01',
	earliestSession: '2024-01-01'
};

const mockComparisonData = {
	project: mockProject,
	totalTokens: 8000,
	estimatedCost: 1.5,
	dateRange: 'Jan - Jun 2024',
	color: '#3b82f6'
};

describe('ComparisonOverviewTable Component', () => {
	let ComparisonOverviewTable: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/ComparisonOverviewTable.svelte');
		ComparisonOverviewTable = mod.default;
	});

	it('should render Overview heading', () => {
		render(ComparisonOverviewTable, { props: { data: [] } });
		expect(screen.getByText('Overview')).toBeInTheDocument();
	});

	it('should render Metric column header', () => {
		render(ComparisonOverviewTable, { props: { data: [mockComparisonData] } });
		expect(screen.getByText('Metric')).toBeInTheDocument();
	});

	it('should render metric rows with data', () => {
		render(ComparisonOverviewTable, { props: { data: [mockComparisonData] } });
		expect(screen.getByText('Sessions')).toBeInTheDocument();
		expect(screen.getByText('Total Tokens')).toBeInTheDocument();
		expect(screen.getByText('Input Tokens')).toBeInTheDocument();
		expect(screen.getByText('Output Tokens')).toBeInTheDocument();
		expect(screen.getByText('Cache Tokens')).toBeInTheDocument();
		expect(screen.getByText('Est. Cost')).toBeInTheDocument();
		expect(screen.getByText('Models')).toBeInTheDocument();
		expect(screen.getByText('Date Range')).toBeInTheDocument();
	});
});

describe('ComparisonProjectSelector Component', () => {
	let ComparisonProjectSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/ComparisonProjectSelector.svelte');
		ComparisonProjectSelector = mod.default;
	});

	it('should render select header with count', () => {
		render(ComparisonProjectSelector, { props: { projects: [] } });
		expect(screen.getByText(/Select Projects/)).toBeInTheDocument();
	});

	it('should render projects', () => {
		render(ComparisonProjectSelector, { props: { projects: [mockProject] } });
		expect(screen.getByText('user/test-project')).toBeInTheDocument();
	});

	it('should show session count and token count', () => {
		render(ComparisonProjectSelector, { props: { projects: [mockProject] } });
		expect(screen.getByText('10 sessions')).toBeInTheDocument();
		expect(screen.getByText('8000 tokens')).toBeInTheDocument();
	});
});

describe('CostComparisonChart Component', () => {
	let CostComparisonChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/CostComparisonChart.svelte');
		CostComparisonChart = mod.default;
	});

	it('should render heading', () => {
		render(CostComparisonChart, { props: { data: [] } });
		expect(screen.getByText('Cost Comparison')).toBeInTheDocument();
	});

	it('should show empty state message with no data', () => {
		render(CostComparisonChart, { props: { data: [] } });
		expect(screen.getByText('Select projects to compare')).toBeInTheDocument();
	});

	it('should render SVG chart with data', () => {
		const { container } = render(CostComparisonChart, { props: { data: [mockComparisonData] } });
		expect(container.querySelector('svg')).toBeInTheDocument();
	});
});

describe('TokenComparisonChart Component', () => {
	let TokenComparisonChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/TokenComparisonChart.svelte');
		TokenComparisonChart = mod.default;
	});

	it('should render heading', () => {
		render(TokenComparisonChart, { props: { data: [] } });
		expect(screen.getByText('Token Comparison')).toBeInTheDocument();
	});

	it('should show empty state with no data', () => {
		render(TokenComparisonChart, { props: { data: [] } });
		expect(screen.getByText('Select projects to compare')).toBeInTheDocument();
	});

	it('should render legend items', () => {
		render(TokenComparisonChart, { props: { data: [] } });
		expect(screen.getByText('Input')).toBeInTheDocument();
		expect(screen.getByText('Output')).toBeInTheDocument();
		expect(screen.getByText('Cache Read')).toBeInTheDocument();
		expect(screen.getByText('Cache Write')).toBeInTheDocument();
	});

	it('should render SVG chart with data', () => {
		const { container } = render(TokenComparisonChart, { props: { data: [mockComparisonData] } });
		expect(container.querySelector('svg')).toBeInTheDocument();
	});
});

describe('ModelMixComparison Component', () => {
	let ModelMixComparison: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/ModelMixComparison.svelte');
		ModelMixComparison = mod.default;
	});

	it('should render heading', () => {
		render(ModelMixComparison, { props: { data: [], allModels: [] } });
		expect(screen.getByText('Model Mix')).toBeInTheDocument();
	});

	it('should show empty state with no data', () => {
		render(ModelMixComparison, { props: { data: [], allModels: [] } });
		expect(screen.getByText('Select projects to compare')).toBeInTheDocument();
	});

	it('should show model pills when data present', () => {
		render(ModelMixComparison, {
			props: { data: [mockComparisonData], allModels: ['claude-3-sonnet'] }
		});
		expect(screen.getAllByText('claude-3-sonnet').length).toBeGreaterThan(0);
	});

	it('should show No models recorded when project has no models', () => {
		const noModelData = { ...mockComparisonData, project: { ...mockProject, modelsUsed: [] } };
		render(ModelMixComparison, {
			props: { data: [noModelData], allModels: [] }
		});
		expect(screen.getByText('No models recorded')).toBeInTheDocument();
	});

	it('should show Summary section when models exist', () => {
		render(ModelMixComparison, {
			props: { data: [mockComparisonData], allModels: ['claude-3-sonnet'] }
		});
		expect(screen.getByText('Summary')).toBeInTheDocument();
	});
});

describe('ToolUsageComparisonChart Component', () => {
	let ToolUsageComparisonChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/comparison/ToolUsageComparisonChart.svelte');
		ToolUsageComparisonChart = mod.default;
	});

	it('should render heading', () => {
		render(ToolUsageComparisonChart, { props: { data: [], tools: [] } });
		expect(screen.getByText('Tool Usage (Top 10)')).toBeInTheDocument();
	});

	it('should show empty state with no tools', () => {
		render(ToolUsageComparisonChart, { props: { data: [], tools: [] } });
		expect(screen.getByText('No tool usage data')).toBeInTheDocument();
	});

	it('should render SVG chart with data', () => {
		const { container } = render(ToolUsageComparisonChart, {
			props: { data: [mockComparisonData], tools: ['Read', 'Edit'] }
		});
		expect(container.querySelector('svg')).toBeInTheDocument();
	});
});
