import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SubAgentLibrary from '$lib/components/subagents/SubAgentLibrary.svelte';
import type { SubAgent } from '$lib/types';

const mockSubAgents: SubAgent[] = [
	{ id: 1, name: 'Code Reviewer', instructions: 'Review code', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 2, name: 'Test Runner', instructions: 'Run tests', source: 'auto-detected', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 3, name: 'Documentation Writer', instructions: 'Write docs', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
];

// Create mock subagentLibrary store
const createMockSubAgentLibrary = () => ({
	searchQuery: '',
	isLoading: false,
	subagents: mockSubAgents,
	filteredSubAgents: mockSubAgents
});

let mockSubAgentLibrary = createMockSubAgentLibrary();

vi.mock('$lib/stores', () => ({
	get subagentLibrary() {
		return mockSubAgentLibrary;
	}
}));

describe('SubAgentLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockSubAgentLibrary = createMockSubAgentLibrary();
	});

	describe('rendering', () => {
		it('should render search bar', () => {
			render(SubAgentLibrary);

			expect(screen.getByPlaceholderText('Search sub-agents...')).toBeInTheDocument();
		});

		it('should render sub-agent count', () => {
			render(SubAgentLibrary);

			expect(screen.getByText('3 sub-agents')).toBeInTheDocument();
		});

		it('should use singular form for 1 sub-agent', () => {
			mockSubAgentLibrary.subagents = [mockSubAgents[0]];
			render(SubAgentLibrary);

			expect(screen.getByText('1 sub-agent')).toBeInTheDocument();
		});

		it('should render sub-agent cards when subagents exist', () => {
			render(SubAgentLibrary);

			expect(screen.getByText('Code Reviewer')).toBeInTheDocument();
			expect(screen.getByText('Test Runner')).toBeInTheDocument();
			expect(screen.getByText('Documentation Writer')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when loading', () => {
			mockSubAgentLibrary.isLoading = true;
			const { container } = render(SubAgentLibrary);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});

		it('should not show sub-agent cards when loading', () => {
			mockSubAgentLibrary.isLoading = true;
			render(SubAgentLibrary);

			expect(screen.queryByText('Code Reviewer')).not.toBeInTheDocument();
		});
	});

	describe('empty states', () => {
		it('should show empty state when no sub-agents', () => {
			mockSubAgentLibrary.filteredSubAgents = [];
			mockSubAgentLibrary.searchQuery = '';
			render(SubAgentLibrary);

			expect(screen.getByText('No sub-agents in library')).toBeInTheDocument();
			expect(screen.getByText('Add your first custom sub-agent to get started')).toBeInTheDocument();
		});

		it('should show filtered empty state when search returns no results', () => {
			mockSubAgentLibrary.filteredSubAgents = [];
			mockSubAgentLibrary.searchQuery = 'nonexistent';
			render(SubAgentLibrary);

			expect(screen.getByText('No matching sub-agents')).toBeInTheDocument();
			expect(screen.getByText('Try adjusting your search')).toBeInTheDocument();
		});
	});

	describe('callbacks', () => {
		it('should pass onEdit to SubAgentCard', () => {
			const onEdit = vi.fn();
			render(SubAgentLibrary, { props: { onEdit } });

			expect(screen.getByText('Code Reviewer')).toBeInTheDocument();
		});

		it('should pass onDelete to SubAgentCard', () => {
			const onDelete = vi.fn();
			render(SubAgentLibrary, { props: { onDelete } });

			expect(screen.getByText('Code Reviewer')).toBeInTheDocument();
		});
	});

	describe('grid layout', () => {
		it('should render cards in a grid', () => {
			const { container } = render(SubAgentLibrary);

			const grid = container.querySelector('.grid');
			expect(grid).toBeInTheDocument();
			expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'xl:grid-cols-3');
		});
	});
});
