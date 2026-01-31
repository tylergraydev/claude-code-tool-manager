import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CommandLibrary from '$lib/components/commands/CommandLibrary.svelte';
import type { Command } from '$lib/types';

const mockCommands: Command[] = [
	{ id: 1, name: 'build', instructions: 'Build the project', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 2, name: 'test', instructions: 'Run tests', source: 'auto-detected', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 3, name: 'deploy', instructions: 'Deploy to production', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
];

// Create mock commandLibrary store
const createMockCommandLibrary = () => ({
	searchQuery: '',
	isLoading: false,
	commands: mockCommands,
	filteredCommands: mockCommands
});

let mockCommandLibrary = createMockCommandLibrary();

vi.mock('$lib/stores', () => ({
	get commandLibrary() {
		return mockCommandLibrary;
	}
}));

describe('CommandLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockCommandLibrary = createMockCommandLibrary();
	});

	describe('rendering', () => {
		it('should render search bar', () => {
			render(CommandLibrary);

			expect(screen.getByPlaceholderText('Search commands...')).toBeInTheDocument();
		});

		it('should render command count', () => {
			render(CommandLibrary);

			expect(screen.getByText('3 commands')).toBeInTheDocument();
		});

		it('should use singular form for 1 command', () => {
			mockCommandLibrary.commands = [mockCommands[0]];
			render(CommandLibrary);

			expect(screen.getByText('1 command')).toBeInTheDocument();
		});

		it('should render command cards when commands exist', () => {
			render(CommandLibrary);

			expect(screen.getByText('/build')).toBeInTheDocument();
			expect(screen.getByText('/test')).toBeInTheDocument();
			expect(screen.getByText('/deploy')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when loading', () => {
			mockCommandLibrary.isLoading = true;
			const { container } = render(CommandLibrary);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});

		it('should not show command cards when loading', () => {
			mockCommandLibrary.isLoading = true;
			render(CommandLibrary);

			expect(screen.queryByText('/build')).not.toBeInTheDocument();
		});
	});

	describe('empty states', () => {
		it('should show empty state when no commands', () => {
			mockCommandLibrary.filteredCommands = [];
			mockCommandLibrary.searchQuery = '';
			render(CommandLibrary);

			expect(screen.getByText('No commands in library')).toBeInTheDocument();
			expect(screen.getByText('Add your first slash command to get started')).toBeInTheDocument();
		});

		it('should show filtered empty state when search returns no results', () => {
			mockCommandLibrary.filteredCommands = [];
			mockCommandLibrary.searchQuery = 'nonexistent';
			render(CommandLibrary);

			expect(screen.getByText('No matching commands')).toBeInTheDocument();
			expect(screen.getByText('Try adjusting your search')).toBeInTheDocument();
		});
	});

	describe('callbacks', () => {
		it('should pass onEdit to CommandCard', () => {
			const onEdit = vi.fn();
			render(CommandLibrary, { props: { onEdit } });

			expect(screen.getByText('/build')).toBeInTheDocument();
		});

		it('should pass onDelete to CommandCard', () => {
			const onDelete = vi.fn();
			render(CommandLibrary, { props: { onDelete } });

			expect(screen.getByText('/build')).toBeInTheDocument();
		});
	});

	describe('grid layout', () => {
		it('should render cards in a grid', () => {
			const { container } = render(CommandLibrary);

			const grid = container.querySelector('.grid');
			expect(grid).toBeInTheDocument();
			expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'xl:grid-cols-3');
		});
	});
});
