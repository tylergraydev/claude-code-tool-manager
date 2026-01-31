import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SkillLibrary from '$lib/components/skills/SkillLibrary.svelte';
import type { Skill } from '$lib/types';

const mockSkills: Skill[] = [
	{ id: 1, name: 'code-review', instructions: 'Review code', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 2, name: 'commit', instructions: 'Commit changes', source: 'auto-detected', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
	{ id: 3, name: 'test-runner', instructions: 'Run tests', source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
];

// Create mock skillLibrary store
const createMockSkillLibrary = () => ({
	searchQuery: '',
	isLoading: false,
	skills: mockSkills,
	filteredSkills: mockSkills
});

let mockSkillLibrary = createMockSkillLibrary();

vi.mock('$lib/stores', () => ({
	get skillLibrary() {
		return mockSkillLibrary;
	}
}));

describe('SkillLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockSkillLibrary = createMockSkillLibrary();
	});

	describe('rendering', () => {
		it('should render search bar', () => {
			render(SkillLibrary);

			expect(screen.getByPlaceholderText('Search skills...')).toBeInTheDocument();
		});

		it('should render skill count', () => {
			render(SkillLibrary);

			expect(screen.getByText('3 skills')).toBeInTheDocument();
		});

		it('should use singular form for 1 skill', () => {
			mockSkillLibrary.skills = [mockSkills[0]];
			render(SkillLibrary);

			expect(screen.getByText('1 skill')).toBeInTheDocument();
		});

		it('should render skill cards when skills exist', () => {
			render(SkillLibrary);

			expect(screen.getByText('code-review')).toBeInTheDocument();
			expect(screen.getByText('commit')).toBeInTheDocument();
			expect(screen.getByText('test-runner')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when loading', () => {
			mockSkillLibrary.isLoading = true;
			const { container } = render(SkillLibrary);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});

		it('should not show skill cards when loading', () => {
			mockSkillLibrary.isLoading = true;
			render(SkillLibrary);

			expect(screen.queryByText('code-review')).not.toBeInTheDocument();
		});
	});

	describe('empty states', () => {
		it('should show empty state when no skills', () => {
			mockSkillLibrary.filteredSkills = [];
			mockSkillLibrary.searchQuery = '';
			render(SkillLibrary);

			expect(screen.getByText('No skills in library')).toBeInTheDocument();
			expect(screen.getByText('Add your first agent skill to get started')).toBeInTheDocument();
		});

		it('should show filtered empty state when search returns no results', () => {
			mockSkillLibrary.filteredSkills = [];
			mockSkillLibrary.searchQuery = 'nonexistent';
			render(SkillLibrary);

			expect(screen.getByText('No matching skills')).toBeInTheDocument();
			expect(screen.getByText('Try adjusting your search')).toBeInTheDocument();
		});
	});

	describe('callbacks', () => {
		it('should pass onEdit to SkillCard', () => {
			const onEdit = vi.fn();
			render(SkillLibrary, { props: { onEdit } });

			expect(screen.getByText('code-review')).toBeInTheDocument();
		});

		it('should pass onDelete to SkillCard', () => {
			const onDelete = vi.fn();
			render(SkillLibrary, { props: { onDelete } });

			expect(screen.getByText('code-review')).toBeInTheDocument();
		});
	});

	describe('grid layout', () => {
		it('should render cards in a grid', () => {
			const { container } = render(SkillLibrary);

			const grid = container.querySelector('.grid');
			expect(grid).toBeInTheDocument();
			expect(grid).toHaveClass('grid-cols-1', 'md:grid-cols-2', 'xl:grid-cols-3');
		});
	});
});
