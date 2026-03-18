import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	mcpLibrary: { load: vi.fn(), mcps: [] },
	projectsStore: { loadProjects: vi.fn(), loadGlobalMcps: vi.fn(), projects: [] },
	notifications: { success: vi.fn(), error: vi.fn() },
	sessionStore: {
		projects: [],
		isLoading: false,
		isLoadingProjects: false,
		load: vi.fn(),
		loadProjects: vi.fn()
	}
}));

vi.mock('$app/stores', () => ({
	page: {
		subscribe: vi.fn((cb: any) => {
			cb({ url: new URL('http://localhost/') });
			return () => {};
		})
	}
}));

vi.mock('@tauri-apps/api/app', () => ({
	getVersion: vi.fn().mockResolvedValue('3.2.4')
}));

vi.mock('$lib/types/usage', () => ({
	estimateSessionCost: vi.fn().mockReturnValue(0),
	formatCost: vi.fn().mockReturnValue('$0.00'),
	formatCompactNumber: vi.fn().mockReturnValue('0')
}));

describe('Header Component', () => {
	let Header: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/layout/Header.svelte');
		Header = mod.default;
	});

	it('should render title', () => {
		render(Header, { props: { title: 'Test Page' } });
		expect(screen.getByText('Test Page')).toBeInTheDocument();
	});

	it('should render subtitle when provided', () => {
		render(Header, { props: { title: 'Page', subtitle: 'Sub info' } });
		expect(screen.getByText('Sub info')).toBeInTheDocument();
	});

	it('should not render subtitle when not provided', () => {
		render(Header, { props: { title: 'Page' } });
		expect(screen.queryByText('Sub info')).not.toBeInTheDocument();
	});

	it('should have refresh button', () => {
		render(Header, { props: { title: 'Page' } });
		expect(screen.getByLabelText('Refresh data')).toBeInTheDocument();
	});

	it('should have theme toggle button', () => {
		render(Header, { props: { title: 'Page' } });
		const themeBtn = screen.getByLabelText(/Switch to/);
		expect(themeBtn).toBeInTheDocument();
	});
});

describe('TodayUsageWidget Component', () => {
	let TodayUsageWidget: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/layout/TodayUsageWidget.svelte');
		TodayUsageWidget = mod.default;
	});

	it('should render without data (empty)', () => {
		render(TodayUsageWidget);
		// When no projects, widget should not show content
		expect(screen.queryByText('Usage')).not.toBeInTheDocument();
	});

	it('should render usage widget when projects exist', async () => {
		const { sessionStore } = await import('$lib/stores');
		(sessionStore as any).projects = [
			{
				folderName: 'proj',
				inferredPath: '/proj',
				sessionCount: 5,
				totalInputTokens: 1000,
				totalOutputTokens: 500,
				totalCacheReadTokens: 0,
				totalCacheCreationTokens: 0,
				modelsUsed: ['sonnet'],
				toolUsage: {},
				latestSession: null,
				earliestSession: null
			}
		];
		render(TodayUsageWidget);
		expect(screen.getByText('Usage')).toBeInTheDocument();
		expect(screen.getByText('Projects')).toBeInTheDocument();
		expect(screen.getByText('Sessions')).toBeInTheDocument();
		expect(screen.getByText('Cost')).toBeInTheDocument();
		(sessionStore as any).projects = [];
	});
});

describe('Sidebar Component', () => {
	let Sidebar: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/layout/Sidebar.svelte');
		Sidebar = mod.default;
	});

	it('should render Claude Code heading', () => {
		render(Sidebar);
		expect(screen.getByText('Claude Code')).toBeInTheDocument();
	});

	it('should render Tool Manager subtitle', () => {
		render(Sidebar);
		expect(screen.getByText('Tool Manager')).toBeInTheDocument();
	});

	it('should render navigation groups', () => {
		render(Sidebar);
		expect(screen.getByText('Core')).toBeInTheDocument();
		expect(screen.getByText('Tools')).toBeInTheDocument();
		expect(screen.getByText('Configure')).toBeInTheDocument();
		expect(screen.getAllByText('Insights').length).toBeGreaterThan(0);
	});

	it('should render nav items', () => {
		render(Sidebar);
		expect(screen.getByText('Dashboard')).toBeInTheDocument();
		expect(screen.getByText('Projects')).toBeInTheDocument();
		expect(screen.getByText('MCPs')).toBeInTheDocument();
		expect(screen.getByText('Skills')).toBeInTheDocument();
		expect(screen.getByText('Commands')).toBeInTheDocument();
		expect(screen.getByText('Hooks')).toBeInTheDocument();
		expect(screen.getByText('Profiles')).toBeInTheDocument();
		expect(screen.getByText('Permissions')).toBeInTheDocument();
		expect(screen.getByText('Memory')).toBeInTheDocument();
		expect(screen.getByText('Analytics')).toBeInTheDocument();
		expect(screen.getByText('Sessions')).toBeInTheDocument();
	});

	it('should render Settings link', () => {
		render(Sidebar);
		expect(screen.getByText('Settings')).toBeInTheDocument();
	});

	it('should render Collapse button', () => {
		render(Sidebar);
		expect(screen.getByText('Collapse')).toBeInTheDocument();
	});
});

describe('Layout index.ts exports', () => {
	let layoutExports: any;

	beforeAll(async () => {
		layoutExports = await import('$lib/components/layout');
	});

	it('should export all layout components', () => {
		expect(layoutExports.Sidebar).toBeDefined();
		expect(layoutExports.Header).toBeDefined();
	});
});
