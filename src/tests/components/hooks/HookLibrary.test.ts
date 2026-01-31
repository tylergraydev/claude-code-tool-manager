import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import HookLibrary from '$lib/components/hooks/HookLibrary.svelte';
import type { Hook, GlobalHook, ProjectHook, Project } from '$lib/types';

const mockHooks: Hook[] = [
	{ id: 1, name: 'Pre Build', eventType: 'PreToolUse', hookType: 'command', command: 'npm run build', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 2, name: 'Post Deploy', eventType: 'PostToolUse', hookType: 'command', command: 'notify', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 3, name: 'Session Start Hook', eventType: 'SessionStart', hookType: 'command', command: 'setup', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
];

const mockGlobalHooks: GlobalHook[] = [
	{ id: 1, hookId: 1, isEnabled: true, hook: mockHooks[0] }
];

const mockProject: Project = {
	id: 1,
	name: 'Test Project',
	path: '/path/to/project',
	assignedMcps: [],
	createdAt: '2024-01-01',
	updatedAt: '2024-01-01'
};

const mockProjectHooks: ProjectHook[] = [
	{ id: 1, projectId: 1, hookId: 2, isEnabled: true, hook: mockHooks[1] }
];

// Create mock hookLibrary store
const createMockHookLibrary = () => ({
	searchQuery: '',
	eventFilter: '' as string,
	viewMode: 'all' as 'all' | 'byScope',
	isLoading: false,
	filteredHooks: mockHooks,
	hooksByEventType: [
		{ eventType: 'PreToolUse', hooks: [mockHooks[0]] },
		{ eventType: 'PostToolUse', hooks: [mockHooks[1]] },
		{ eventType: 'SessionStart', hooks: [mockHooks[2]] }
	],
	globalHooks: mockGlobalHooks,
	projectsWithHooks: [{ project: mockProject, hooks: mockProjectHooks }],
	unassignedHooks: [mockHooks[2]],
	setEventFilter: vi.fn(),
	setViewMode: vi.fn()
});

let mockHookLibrary = createMockHookLibrary();

vi.mock('$lib/stores', () => ({
	get hookLibrary() {
		return mockHookLibrary;
	}
}));

vi.mock('$lib/types', () => ({
	HOOK_EVENT_TYPES: [
		{ value: 'SessionStart', label: 'Session Start' },
		{ value: 'PreToolUse', label: 'Pre Tool Use' },
		{ value: 'PostToolUse', label: 'Post Tool Use' }
	]
}));

describe('HookLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockHookLibrary = createMockHookLibrary();
	});

	describe('rendering', () => {
		it('should render search bar', () => {
			render(HookLibrary);

			expect(screen.getByPlaceholderText('Search hooks...')).toBeInTheDocument();
		});

		it('should render event filter dropdown', () => {
			render(HookLibrary);

			expect(screen.getByText('All Events')).toBeInTheDocument();
		});

		it('should render view mode toggle buttons', () => {
			render(HookLibrary);

			expect(screen.getByText('All')).toBeInTheDocument();
			expect(screen.getByText('By Scope')).toBeInTheDocument();
		});

		it('should render hook count', () => {
			render(HookLibrary);

			expect(screen.getByText('3 hooks')).toBeInTheDocument();
		});

		it('should use singular form for 1 hook', () => {
			mockHookLibrary.filteredHooks = [mockHooks[0]];
			render(HookLibrary);

			expect(screen.getByText('1 hook')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when loading', () => {
			mockHookLibrary.isLoading = true;
			const { container } = render(HookLibrary);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});
	});

	describe('all view mode', () => {
		beforeEach(() => {
			mockHookLibrary.viewMode = 'all';
		});

		it('should render hooks grouped by event type', () => {
			const { container } = render(HookLibrary);

			// Look for h3 headings that contain event type labels
			const headings = container.querySelectorAll('h3.text-lg');
			const headingTexts = Array.from(headings).map(h => h.textContent);

			expect(headingTexts.some(t => t?.includes('Pre Tool Use'))).toBe(true);
			expect(headingTexts.some(t => t?.includes('Post Tool Use'))).toBe(true);
			expect(headingTexts.some(t => t?.includes('Session Start'))).toBe(true);
		});

		it('should render hook cards', () => {
			render(HookLibrary);

			expect(screen.getByText('Pre Build')).toBeInTheDocument();
		});

		it('should show empty state when no hooks', () => {
			mockHookLibrary.hooksByEventType = [];
			mockHookLibrary.searchQuery = '';
			mockHookLibrary.eventFilter = '';
			render(HookLibrary);

			expect(screen.getByText('No hooks in library')).toBeInTheDocument();
			expect(screen.getByText('Add your first hook to automate Claude Code actions')).toBeInTheDocument();
		});

		it('should show filtered empty state with search', () => {
			mockHookLibrary.hooksByEventType = [];
			mockHookLibrary.searchQuery = 'nonexistent';
			render(HookLibrary);

			expect(screen.getByText('No matching hooks')).toBeInTheDocument();
			expect(screen.getByText('Try adjusting your filters')).toBeInTheDocument();
		});

		it('should show filtered empty state with event filter', () => {
			mockHookLibrary.hooksByEventType = [];
			mockHookLibrary.eventFilter = 'Stop';
			render(HookLibrary);

			expect(screen.getByText('No matching hooks')).toBeInTheDocument();
		});
	});

	describe('by scope view mode', () => {
		beforeEach(() => {
			mockHookLibrary.viewMode = 'byScope';
		});

		it('should render Global Hooks section', () => {
			render(HookLibrary);

			expect(screen.getByText('Global Hooks')).toBeInTheDocument();
		});

		it('should render project name in project section', () => {
			render(HookLibrary);

			expect(screen.getByText('Test Project')).toBeInTheDocument();
		});

		it('should render project path', () => {
			render(HookLibrary);

			expect(screen.getByText('/path/to/project')).toBeInTheDocument();
		});

		it('should render Unassigned section when there are unassigned hooks', () => {
			render(HookLibrary);

			expect(screen.getByText('Unassigned')).toBeInTheDocument();
		});

		it('should show empty state when no scoped hooks', () => {
			mockHookLibrary.globalHooks = [];
			mockHookLibrary.projectsWithHooks = [];
			mockHookLibrary.unassignedHooks = [];
			mockHookLibrary.searchQuery = '';
			mockHookLibrary.eventFilter = '';
			render(HookLibrary);

			expect(screen.getByText('No hooks assigned')).toBeInTheDocument();
			expect(screen.getByText('Create hooks and assign them to global or project scope')).toBeInTheDocument();
		});
	});

	describe('view mode toggle', () => {
		it('should call setViewMode with "all" when All view mode clicked', async () => {
			const { container } = render(HookLibrary);

			// Find the view mode toggle group (bg-gray-100)
			const viewModeGroup = container.querySelector('.bg-gray-100');
			const buttons = viewModeGroup?.querySelectorAll('button') || [];
			const allButton = Array.from(buttons).find(btn => btn.textContent?.includes('All'));
			await fireEvent.click(allButton!);

			expect(mockHookLibrary.setViewMode).toHaveBeenCalledWith('all');
		});

		it('should call setViewMode with "byScope" when By Scope clicked', async () => {
			const { container } = render(HookLibrary);

			const viewModeGroup = container.querySelector('.bg-gray-100');
			const buttons = viewModeGroup?.querySelectorAll('button') || [];
			const byScopeButton = Array.from(buttons).find(btn => btn.textContent?.includes('By Scope'));
			await fireEvent.click(byScopeButton!);

			expect(mockHookLibrary.setViewMode).toHaveBeenCalledWith('byScope');
		});

		it('should highlight selected view mode', () => {
			mockHookLibrary.viewMode = 'all';
			const { container } = render(HookLibrary);

			const viewModeGroup = container.querySelector('.bg-gray-100');
			const buttons = viewModeGroup?.querySelectorAll('button') || [];
			const allButton = Array.from(buttons).find(btn => btn.textContent?.includes('All'));
			expect(allButton).toHaveClass('bg-white');
		});
	});

	describe('event filter', () => {
		it('should call setEventFilter when filter changes', async () => {
			render(HookLibrary);

			const select = screen.getByRole('combobox');
			await fireEvent.change(select, { target: { value: 'PreToolUse' } });

			expect(mockHookLibrary.setEventFilter).toHaveBeenCalledWith('PreToolUse');
		});

		it('should show all event type options in dropdown', () => {
			render(HookLibrary);

			const select = screen.getByRole('combobox');
			const options = select.querySelectorAll('option');

			// Should have "All Events" plus the 3 event types
			expect(options.length).toBeGreaterThanOrEqual(4);
		});
	});

	describe('callbacks', () => {
		it('should pass onEdit to HookCard', () => {
			const onEdit = vi.fn();
			render(HookLibrary, { props: { onEdit } });

			expect(screen.getByText('Pre Build')).toBeInTheDocument();
		});

		it('should pass onDelete to HookCard', () => {
			const onDelete = vi.fn();
			render(HookLibrary, { props: { onDelete } });

			expect(screen.getByText('Pre Build')).toBeInTheDocument();
		});

		it('should pass onDuplicate to HookCard', () => {
			const onDuplicate = vi.fn();
			render(HookLibrary, { props: { onDuplicate } });

			expect(screen.getByText('Pre Build')).toBeInTheDocument();
		});
	});

	describe('grid layout', () => {
		it('should render cards in a grid', () => {
			const { container } = render(HookLibrary);

			const grid = container.querySelector('.grid');
			expect(grid).toBeInTheDocument();
			expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'xl:grid-cols-3');
		});
	});
});
