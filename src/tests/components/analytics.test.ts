import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/types/usage', () => ({
	formatCompactNumber: vi.fn((n: number) => {
		if (n >= 1000) return `${(n / 1000).toFixed(0)}k`;
		return String(n);
	}),
	formatDuration: vi.fn((ms: number) => {
		const m = Math.floor(ms / 60000);
		return `${m}m`;
	}),
	formatCost: vi.fn((n: number) => `$${n.toFixed(2)}`),
	estimateSessionCost: vi.fn(() => 0),
	getModelColor: vi.fn((m: string) => '#3b82f6'),
	formatModelName: vi.fn((m: string) => m.split('/').pop() || m),
	estimateModelCost: vi.fn(() => 0)
}));

vi.mock('$lib/types/insights', () => ({
	OUTCOME_LABELS: { success: 'Success', partial_success: 'Partial Success', failure: 'Failure' },
	OUTCOME_COLORS: { success: '#22c55e', partial_success: '#f59e0b', failure: '#ef4444' },
	HELPFULNESS_LABELS: { very_helpful: 'Very Helpful', somewhat_helpful: 'Somewhat Helpful', not_helpful: 'Not Helpful' },
	HELPFULNESS_COLORS: { very_helpful: '#22c55e', somewhat_helpful: '#f59e0b', not_helpful: '#ef4444' },
	SESSION_TYPE_LABELS: { coding: 'Coding', debugging: 'Debugging', exploration: 'Exploration' },
	FRICTION_LABELS: { slow_response: 'Slow Response', context_limit: 'Context Limit' }
}));

vi.mock('$lib/stores', () => ({
	usageStore: {
		overview: null,
		dailyActivity: [],
		dailyTokens: [],
		dailyCosts: [],
		modelBreakdown: [],
		peakHours: [],
		isLoading: false,
		load: vi.fn()
	},
	insightsStore: {
		report: null,
		isLoading: false,
		load: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

// ──────────────────────────────────────────────────────────
// OverviewCards
// ──────────────────────────────────────────────────────────
describe('OverviewCards Component', () => {
	let OverviewCards: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/OverviewCards.svelte');
		OverviewCards = mod.default;
	});

	it('should render all metric cards with data', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 42,
				totalMessages: 100,
				totalToolCalls: 50,
				firstSessionDate: '2024-01-01',
				longestSession: null,
				lastComputedDate: '2024-06-01',
				totalCostUSD: 15.5
			}
		});
		expect(screen.getByText('42')).toBeInTheDocument();
		expect(screen.getByText('100')).toBeInTheDocument();
		expect(screen.getByText('50')).toBeInTheDocument();
	});

	it('should show 0 for null session and message counts', () => {
		render(OverviewCards, {
			props: {
				totalSessions: null,
				totalMessages: null,
				totalToolCalls: 0,
				firstSessionDate: null,
				longestSession: null,
				lastComputedDate: null
			}
		});
		const zeros = screen.getAllByText('0');
		expect(zeros.length).toBeGreaterThanOrEqual(2);
	});

	it('should render label cards', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 1,
				totalMessages: 1,
				totalToolCalls: 1,
				firstSessionDate: '2024-01-01',
				longestSession: null,
				lastComputedDate: null
			}
		});
		expect(screen.getByText('Total Sessions')).toBeInTheDocument();
		expect(screen.getByText('Total Messages')).toBeInTheDocument();
		expect(screen.getByText('Tool Calls')).toBeInTheDocument();
		expect(screen.getByText('First Session')).toBeInTheDocument();
		expect(screen.getByText('Longest Session')).toBeInTheDocument();
	});

	it('should render N/A for longest session when null', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 1,
				totalMessages: 1,
				totalToolCalls: 1,
				firstSessionDate: null,
				longestSession: null,
				lastComputedDate: null
			}
		});
		const naElements = screen.getAllByText('N/A');
		expect(naElements.length).toBeGreaterThan(0);
	});

	it('should render longest session duration when provided', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 1,
				totalMessages: 1,
				totalToolCalls: 1,
				firstSessionDate: null,
				longestSession: { duration: 90000, messageCount: 5 },
				lastComputedDate: null
			}
		});
		// formatDuration(90000) = "1m"
		expect(screen.getByText('1m')).toBeInTheDocument();
		expect(screen.getByText('5 messages')).toBeInTheDocument();
	});

	it('should render Est. API Cost when totalCostUSD is provided', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 1,
				totalMessages: 1,
				totalToolCalls: 1,
				firstSessionDate: null,
				longestSession: null,
				lastComputedDate: null,
				totalCostUSD: 25.5
			}
		});
		expect(screen.getByText('Est. API Cost')).toBeInTheDocument();
		expect(screen.getByText('$25.50')).toBeInTheDocument();
	});

	it('should render last computed date', () => {
		render(OverviewCards, {
			props: {
				totalSessions: 1,
				totalMessages: 1,
				totalToolCalls: 1,
				firstSessionDate: null,
				longestSession: null,
				lastComputedDate: '2024-06-15'
			}
		});
		expect(document.body.textContent).toContain('Last updated:');
	});
});

// ──────────────────────────────────────────────────────────
// DailyActivityChart
// ──────────────────────────────────────────────────────────
describe('DailyActivityChart Component', () => {
	let DailyActivityChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/DailyActivityChart.svelte');
		DailyActivityChart = mod.default;
	});

	it('should render title', () => {
		render(DailyActivityChart, {
			props: { data: [], dateRange: '30d', onDateRangeChange: vi.fn() }
		});
		expect(screen.getByText('Daily Activity')).toBeInTheDocument();
	});

	it('should render empty data message', () => {
		render(DailyActivityChart, {
			props: { data: [], dateRange: '30d', onDateRangeChange: vi.fn() }
		});
		expect(screen.getByText('No activity data available')).toBeInTheDocument();
	});

	it('should render metric selector buttons', () => {
		render(DailyActivityChart, {
			props: { data: [], dateRange: '30d', onDateRangeChange: vi.fn() }
		});
		expect(screen.getByText('Messages')).toBeInTheDocument();
		expect(screen.getByText('Sessions')).toBeInTheDocument();
		expect(screen.getByText('Tool Calls')).toBeInTheDocument();
	});

	it('should render date range buttons', () => {
		render(DailyActivityChart, {
			props: { data: [], dateRange: '30d', onDateRangeChange: vi.fn() }
		});
		expect(screen.getByText('7d')).toBeInTheDocument();
		expect(screen.getByText('30d')).toBeInTheDocument();
		expect(screen.getByText('All')).toBeInTheDocument();
	});

	it('should render chart with data', () => {
		const data = [
			{ date: '2024-01-01', messageCount: 10, sessionCount: 2, toolCallCount: 5 },
			{ date: '2024-01-02', messageCount: 20, sessionCount: 3, toolCallCount: 8 }
		];
		const { container } = render(DailyActivityChart, {
			props: { data, dateRange: '30d', onDateRangeChange: vi.fn() }
		});
		// Should render SVG bars
		const rects = container.querySelectorAll('rect');
		expect(rects.length).toBeGreaterThan(0);
	});
});

// ──────────────────────────────────────────────────────────
// DailyCostChart
// ──────────────────────────────────────────────────────────
describe('DailyCostChart Component', () => {
	let DailyCostChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/DailyCostChart.svelte');
		DailyCostChart = mod.default;
	});

	it('should render title', () => {
		render(DailyCostChart, { props: { data: [], models: [] } });
		expect(screen.getByText('Daily Cost')).toBeInTheDocument();
	});

	it('should render empty data message', () => {
		render(DailyCostChart, { props: { data: [], models: [] } });
		expect(screen.getByText('No cost data available')).toBeInTheDocument();
	});

	it('should render chart with data', () => {
		const data = [
			{ date: '2024-01-01', total: 1.5, costByModel: { 'claude-3-opus': 1.5 } }
		];
		const { container } = render(DailyCostChart, {
			props: { data, models: ['claude-3-opus'] }
		});
		const rects = container.querySelectorAll('rect');
		expect(rects.length).toBeGreaterThan(0);
	});

	it('should render model legend', () => {
		const data = [
			{ date: '2024-01-01', total: 1.0, costByModel: { 'claude-3-opus': 1.0 } }
		];
		render(DailyCostChart, {
			props: { data, models: ['claude-3-opus'] }
		});
		expect(screen.getByText('claude-3-opus')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// DailyTokenChart
// ──────────────────────────────────────────────────────────
describe('DailyTokenChart Component', () => {
	let DailyTokenChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/DailyTokenChart.svelte');
		DailyTokenChart = mod.default;
	});

	it('should render title', () => {
		render(DailyTokenChart, { props: { data: [], models: [] } });
		expect(screen.getByText('Daily Token Usage')).toBeInTheDocument();
	});

	it('should render empty data message', () => {
		render(DailyTokenChart, { props: { data: [], models: [] } });
		expect(screen.getByText('No token data available')).toBeInTheDocument();
	});

	it('should render chart with data', () => {
		const data = [
			{ date: '2024-01-01', tokensByModel: { 'claude-3-opus': 50000 } }
		];
		const { container } = render(DailyTokenChart, {
			props: { data, models: ['claude-3-opus'] }
		});
		const rects = container.querySelectorAll('rect');
		expect(rects.length).toBeGreaterThan(0);
	});

	it('should render model legend', () => {
		render(DailyTokenChart, {
			props: {
				data: [{ date: '2024-01-01', tokensByModel: { 'claude-3-opus': 50000 } }],
				models: ['claude-3-opus']
			}
		});
		expect(screen.getByText('claude-3-opus')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ModelUsageBreakdown
// ──────────────────────────────────────────────────────────
describe('ModelUsageBreakdown Component', () => {
	let ModelUsageBreakdown: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/ModelUsageBreakdown.svelte');
		ModelUsageBreakdown = mod.default;
	});

	it('should render title', () => {
		render(ModelUsageBreakdown, { props: { modelUsage: {} } });
		expect(screen.getByText('Model Usage')).toBeInTheDocument();
	});

	it('should render empty data message when no usage', () => {
		render(ModelUsageBreakdown, { props: { modelUsage: {} } });
		expect(screen.getByText('No model usage data')).toBeInTheDocument();
	});

	it('should render model data when provided', () => {
		render(ModelUsageBreakdown, {
			props: {
				modelUsage: {
					'claude-3-opus': {
						inputTokens: 100000,
						outputTokens: 5000,
						cacheReadInputTokens: 20000,
						cacheCreationInputTokens: 1000,
						costUSD: 5.0
					}
				}
			}
		});
		expect(screen.getByText('total tokens')).toBeInTheDocument();
		// Table headers
		expect(screen.getByText('Input')).toBeInTheDocument();
		expect(screen.getByText('Output')).toBeInTheDocument();
		expect(screen.getByText('Cache Read')).toBeInTheDocument();
		expect(screen.getByText('Cache Write')).toBeInTheDocument();
		expect(screen.getByText('Total')).toBeInTheDocument();
		expect(screen.getByText('Est. Cost')).toBeInTheDocument();
	});

	it('should render donut chart SVG', () => {
		const { container } = render(ModelUsageBreakdown, {
			props: {
				modelUsage: {
					'claude-3-opus': {
						inputTokens: 100000,
						outputTokens: 5000,
						cacheReadInputTokens: 0,
						cacheCreationInputTokens: 0,
						costUSD: 1.0
					}
				}
			}
		});
		const circles = container.querySelectorAll('circle');
		expect(circles.length).toBeGreaterThan(0);
	});
});

// ──────────────────────────────────────────────────────────
// PeakHoursChart
// ──────────────────────────────────────────────────────────
describe('PeakHoursChart Component', () => {
	let PeakHoursChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/PeakHoursChart.svelte');
		PeakHoursChart = mod.default;
	});

	it('should render title', () => {
		render(PeakHoursChart, { props: { hourCounts: new Array(24).fill(0) } });
		expect(screen.getByText('Peak Hours')).toBeInTheDocument();
	});

	it('should render empty data message when all zeros', () => {
		render(PeakHoursChart, { props: { hourCounts: new Array(24).fill(0) } });
		expect(screen.getByText('No hour data available')).toBeInTheDocument();
	});

	it('should render chart with data', () => {
		const hourCounts = new Array(24).fill(0);
		hourCounts[14] = 10;
		hourCounts[15] = 8;
		const { container } = render(PeakHoursChart, { props: { hourCounts } });
		const rects = container.querySelectorAll('rect');
		expect(rects.length).toBeGreaterThan(0);
	});

	it('should show peak hour info', () => {
		const hourCounts = new Array(24).fill(0);
		hourCounts[14] = 10;
		render(PeakHoursChart, { props: { hourCounts } });
		expect(document.body.textContent).toContain('Peak:');
		expect(document.body.textContent).toContain('2:00 PM');
		expect(document.body.textContent).toContain('10 sessions');
	});
});

// ──────────────────────────────────────────────────────────
// CostProjectionsCard
// ──────────────────────────────────────────────────────────
describe('CostProjectionsCard Component', () => {
	let CostProjectionsCard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/CostProjectionsCard.svelte');
		CostProjectionsCard = mod.default;
	});

	it('should render title', () => {
		render(CostProjectionsCard, {
			props: { dailyCosts: [], totalCostUSD: 0 }
		});
		expect(screen.getByText('Cost Projections')).toBeInTheDocument();
	});

	it('should render no cost data when totalCostUSD is 0', () => {
		render(CostProjectionsCard, {
			props: { dailyCosts: [], totalCostUSD: 0 }
		});
		expect(screen.getByText('No cost data')).toBeInTheDocument();
	});

	it('should render projections with data', () => {
		const dailyCosts = [
			{ date: '2024-01-01', total: 2.0, costByModel: {} },
			{ date: '2024-01-02', total: 3.0, costByModel: {} }
		];
		render(CostProjectionsCard, {
			props: { dailyCosts, totalCostUSD: 5.0 }
		});
		expect(screen.getByText('Daily Avg')).toBeInTheDocument();
		expect(screen.getByText('Weekly Est.')).toBeInTheDocument();
		expect(screen.getByText('Monthly Est.')).toBeInTheDocument();
	});

	it('should show flat trend with insufficient data', () => {
		const dailyCosts = [
			{ date: '2024-01-01', total: 2.0, costByModel: {} }
		];
		render(CostProjectionsCard, {
			props: { dailyCosts, totalCostUSD: 2.0 }
		});
		expect(screen.getByText('Spending stable')).toBeInTheDocument();
	});

	it('should show spending trending up when recent > prior', () => {
		const dailyCosts = [
			// Prior period (older)
			{ date: '2024-01-01', total: 1.0, costByModel: {} },
			{ date: '2024-01-02', total: 1.0, costByModel: {} },
			{ date: '2024-01-03', total: 1.0, costByModel: {} },
			{ date: '2024-01-04', total: 1.0, costByModel: {} },
			{ date: '2024-01-05', total: 1.0, costByModel: {} },
			{ date: '2024-01-06', total: 1.0, costByModel: {} },
			{ date: '2024-01-07', total: 1.0, costByModel: {} },
			// Recent period (newer)
			{ date: '2024-01-08', total: 5.0, costByModel: {} },
			{ date: '2024-01-09', total: 5.0, costByModel: {} },
			{ date: '2024-01-10', total: 5.0, costByModel: {} },
			{ date: '2024-01-11', total: 5.0, costByModel: {} },
			{ date: '2024-01-12', total: 5.0, costByModel: {} },
			{ date: '2024-01-13', total: 5.0, costByModel: {} },
			{ date: '2024-01-14', total: 5.0, costByModel: {} }
		];
		render(CostProjectionsCard, {
			props: { dailyCosts, totalCostUSD: 42.0 }
		});
		expect(screen.getByText('Spending trending up')).toBeInTheDocument();
	});

	it('should show spending trending down when recent < prior', () => {
		const dailyCosts = [
			// Prior period (older, higher)
			{ date: '2024-01-01', total: 5.0, costByModel: {} },
			{ date: '2024-01-02', total: 5.0, costByModel: {} },
			{ date: '2024-01-03', total: 5.0, costByModel: {} },
			{ date: '2024-01-04', total: 5.0, costByModel: {} },
			{ date: '2024-01-05', total: 5.0, costByModel: {} },
			{ date: '2024-01-06', total: 5.0, costByModel: {} },
			{ date: '2024-01-07', total: 5.0, costByModel: {} },
			// Recent period (newer, lower)
			{ date: '2024-01-08', total: 1.0, costByModel: {} },
			{ date: '2024-01-09', total: 1.0, costByModel: {} },
			{ date: '2024-01-10', total: 1.0, costByModel: {} },
			{ date: '2024-01-11', total: 1.0, costByModel: {} },
			{ date: '2024-01-12', total: 1.0, costByModel: {} },
			{ date: '2024-01-13', total: 1.0, costByModel: {} },
			{ date: '2024-01-14', total: 1.0, costByModel: {} }
		];
		render(CostProjectionsCard, {
			props: { dailyCosts, totalCostUSD: 42.0 }
		});
		expect(screen.getByText('Spending trending down')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// FrictionTrendsChart
// ──────────────────────────────────────────────────────────
describe('FrictionTrendsChart Component', () => {
	let FrictionTrendsChart: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/FrictionTrendsChart.svelte');
		FrictionTrendsChart = mod.default;
	});

	it('should render title', () => {
		render(FrictionTrendsChart, { props: { facets: [] } });
		expect(screen.getByText('Friction Categories')).toBeInTheDocument();
	});

	it('should render empty state when no friction data', () => {
		render(FrictionTrendsChart, { props: { facets: [] } });
		expect(screen.getByText('No friction data recorded')).toBeInTheDocument();
	});

	it('should render friction bars when data present', () => {
		const facets = [
			{
				sessionId: 's1',
				outcome: 'success',
				frictionCounts: { slow_response: 3, context_limit: 1 },
				claudeHelpfulness: null,
				sessionType: null,
				primarySuccess: null,
				frictionDetail: null,
				goalCategories: {},
				briefSummary: '',
				underlyingGoal: ''
			}
		];
		render(FrictionTrendsChart, { props: { facets } });
		expect(screen.getByText('Slow Response')).toBeInTheDocument();
		expect(screen.getByText('3')).toBeInTheDocument();
		expect(screen.getByText('Context Limit')).toBeInTheDocument();
		expect(screen.getByText('1')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// SessionQualityCards
// ──────────────────────────────────────────────────────────
describe('SessionQualityCards Component', () => {
	let SessionQualityCards: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/SessionQualityCards.svelte');
		SessionQualityCards = mod.default;
	});

	it('should render Session Outcomes and Claude Helpfulness headings', () => {
		render(SessionQualityCards, { props: { facets: [] } });
		expect(screen.getByText('Session Outcomes')).toBeInTheDocument();
		expect(screen.getByText('Claude Helpfulness')).toBeInTheDocument();
	});

	it('should render empty outcome state', () => {
		render(SessionQualityCards, { props: { facets: [] } });
		expect(screen.getByText('No outcome data available')).toBeInTheDocument();
	});

	it('should render empty helpfulness state', () => {
		render(SessionQualityCards, { props: { facets: [] } });
		expect(screen.getByText('No helpfulness data available')).toBeInTheDocument();
	});

	it('should render outcome donut with data', () => {
		const facets = [
			{
				sessionId: 's1',
				outcome: 'success',
				claudeHelpfulness: 'very_helpful',
				frictionCounts: {},
				sessionType: null,
				primarySuccess: null,
				frictionDetail: null,
				goalCategories: {},
				briefSummary: '',
				underlyingGoal: ''
			},
			{
				sessionId: 's2',
				outcome: 'failure',
				claudeHelpfulness: 'not_helpful',
				frictionCounts: {},
				sessionType: null,
				primarySuccess: null,
				frictionDetail: null,
				goalCategories: {},
				briefSummary: '',
				underlyingGoal: ''
			}
		];
		const { container } = render(SessionQualityCards, { props: { facets } });
		// Should render SVG circles for donuts
		const circles = container.querySelectorAll('circle');
		expect(circles.length).toBeGreaterThan(0);
		expect(screen.getByText('Success')).toBeInTheDocument();
		expect(screen.getByText('Failure')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// SessionSummaryList
// ──────────────────────────────────────────────────────────
describe('SessionSummaryList Component', () => {
	let SessionSummaryList: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/SessionSummaryList.svelte');
		SessionSummaryList = mod.default;
	});

	it('should render title', () => {
		render(SessionSummaryList, { props: { facets: [] } });
		expect(screen.getByText('Session Summaries')).toBeInTheDocument();
	});

	it('should render empty state', () => {
		render(SessionSummaryList, { props: { facets: [] } });
		expect(screen.getByText('No session data available')).toBeInTheDocument();
	});

	it('should render session count', () => {
		render(SessionSummaryList, { props: { facets: [] } });
		expect(screen.getByText('0 sessions')).toBeInTheDocument();
	});

	it('should render session entries', () => {
		const facets = [
			{
				sessionId: 'abc123def456',
				outcome: 'success',
				claudeHelpfulness: 'very_helpful',
				frictionCounts: {},
				sessionType: 'coding',
				primarySuccess: 'true',
				frictionDetail: null,
				goalCategories: {},
				briefSummary: 'Fixed a bug in auth',
				underlyingGoal: 'Improve login flow'
			}
		];
		render(SessionSummaryList, { props: { facets } });
		expect(screen.getByText('Fixed a bug in auth')).toBeInTheDocument();
		expect(screen.getByText('1 sessions')).toBeInTheDocument();
	});

	it('should show Untitled session when no summary or goal', () => {
		const facets = [
			{
				sessionId: 'abc123',
				outcome: 'success',
				claudeHelpfulness: null,
				frictionCounts: {},
				sessionType: null,
				primarySuccess: null,
				frictionDetail: null,
				goalCategories: {},
				briefSummary: '',
				underlyingGoal: ''
			}
		];
		render(SessionSummaryList, { props: { facets } });
		expect(screen.getByText('Untitled session')).toBeInTheDocument();
	});

	it('should truncate long session IDs', () => {
		const facets = [
			{
				sessionId: 'abcdefghijklmnop',
				outcome: 'success',
				claudeHelpfulness: null,
				frictionCounts: {},
				sessionType: null,
				primarySuccess: null,
				frictionDetail: null,
				goalCategories: {},
				briefSummary: 'Test',
				underlyingGoal: ''
			}
		];
		render(SessionSummaryList, { props: { facets } });
		expect(screen.getByText('abcdefgh...')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// InsightsReportViewer
// ──────────────────────────────────────────────────────────
describe('InsightsReportViewer Component', () => {
	let InsightsReportViewer: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/analytics/InsightsReportViewer.svelte');
		InsightsReportViewer = mod.default;
	});

	it('should render title', () => {
		render(InsightsReportViewer, {
			props: { htmlContent: '<p>Hello</p>', filePath: '/tmp/report.html' }
		});
		expect(screen.getByText('Insights Report')).toBeInTheDocument();
	});

	it('should render Open in Browser button', () => {
		render(InsightsReportViewer, {
			props: { htmlContent: '<p>Hello</p>', filePath: '/tmp/report.html' }
		});
		expect(screen.getByText('Open in Browser')).toBeInTheDocument();
	});

	it('should render Refresh button when onRefresh provided', () => {
		render(InsightsReportViewer, {
			props: { htmlContent: '<p>Hello</p>', filePath: '/tmp/report.html', onRefresh: vi.fn() }
		});
		expect(screen.getByText('Refresh')).toBeInTheDocument();
	});

	it('should not render Refresh button when onRefresh not provided', () => {
		render(InsightsReportViewer, {
			props: { htmlContent: '<p>Hello</p>', filePath: '/tmp/report.html' }
		});
		expect(screen.queryByText('Refresh')).not.toBeInTheDocument();
	});

	it('should render iframe with title', () => {
		render(InsightsReportViewer, {
			props: { htmlContent: '<p>Hello</p>', filePath: '/tmp/report.html' }
		});
		expect(screen.getByTitle('Claude Code Insights Report')).toBeInTheDocument();
	});
});
