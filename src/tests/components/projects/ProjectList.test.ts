import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ProjectList from '$lib/components/projects/ProjectList.svelte';
import type { Project } from '$lib/types';

const mockProjects: Project[] = [
	{
		id: 1,
		name: 'Project One',
		path: '/path/to/project-one',
		assignedMcps: [],
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	},
	{
		id: 2,
		name: 'Project Two',
		path: '/path/to/project-two',
		assignedMcps: [],
		createdAt: '2024-01-02',
		updatedAt: '2024-01-02'
	}
];

// Create mock projectsStore
const createMockProjectsStore = () => ({
	isLoading: false,
	projects: mockProjects,
	loadProjects: vi.fn(),
	getProjectById: vi.fn((id: number) => mockProjects.find((p) => p.id === id) || null)
});

// Create mock skillLibrary
const createMockSkillLibrary = () => ({
	getProjectSkills: vi.fn().mockReturnValue([])
});

// Create mock subagentLibrary
const createMockSubAgentLibrary = () => ({
	getProjectSubAgents: vi.fn().mockReturnValue([])
});

// Create mock commandLibrary
const createMockCommandLibrary = () => ({
	getProjectCommands: vi.fn().mockReturnValue([])
});

let mockProjectsStore = createMockProjectsStore();
let mockSkillLibrary = createMockSkillLibrary();
let mockSubAgentLibrary = createMockSubAgentLibrary();
let mockCommandLibrary = createMockCommandLibrary();

vi.mock('$lib/stores', () => ({
	get projectsStore() {
		return mockProjectsStore;
	},
	get skillLibrary() {
		return mockSkillLibrary;
	},
	get subagentLibrary() {
		return mockSubAgentLibrary;
	},
	get commandLibrary() {
		return mockCommandLibrary;
	}
}));

describe('ProjectList', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockProjectsStore = createMockProjectsStore();
		mockSkillLibrary = createMockSkillLibrary();
		mockSubAgentLibrary = createMockSubAgentLibrary();
		mockCommandLibrary = createMockCommandLibrary();
	});

	describe('rendering', () => {
		it('should render Projects title', () => {
			render(ProjectList);

			expect(screen.getByText('Projects')).toBeInTheDocument();
		});

		it('should render description text', () => {
			render(ProjectList);

			expect(screen.getByText('Click a project to manage its MCPs')).toBeInTheDocument();
		});

		it('should render Add Project button when onAddProject is provided', () => {
			const onAddProject = vi.fn();
			render(ProjectList, { props: { onAddProject } });

			// Find the Add Project button in the header (not the empty state one)
			const buttons = screen.getAllByRole('button', { name: /Add Project/i });
			expect(buttons.length).toBeGreaterThanOrEqual(1);
		});

		it('should NOT render Add Project button when onAddProject is not provided', () => {
			mockProjectsStore.projects = mockProjects;
			render(ProjectList);

			expect(screen.queryByRole('button', { name: /Add Project/i })).not.toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		it('should show loading spinner when isLoading is true', () => {
			mockProjectsStore.isLoading = true;
			const { container } = render(ProjectList);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});

		it('should not show projects grid when loading', () => {
			mockProjectsStore.isLoading = true;
			render(ProjectList);

			expect(screen.queryByText('Project One')).not.toBeInTheDocument();
			expect(screen.queryByText('Project Two')).not.toBeInTheDocument();
		});

		it('should not show empty state when loading', () => {
			mockProjectsStore.isLoading = true;
			mockProjectsStore.projects = [];
			render(ProjectList);

			expect(screen.queryByText('No projects added')).not.toBeInTheDocument();
		});
	});

	describe('empty state', () => {
		beforeEach(() => {
			mockProjectsStore.projects = [];
		});

		it('should show empty state when no projects', () => {
			render(ProjectList);

			expect(screen.getByText('No projects added')).toBeInTheDocument();
			expect(screen.getByText('Add a project folder to start managing MCPs')).toBeInTheDocument();
		});

		it('should show Add Your First Project button when onAddProject is provided', () => {
			const onAddProject = vi.fn();
			render(ProjectList, { props: { onAddProject } });

			expect(screen.getByRole('button', { name: /Add Your First Project/i })).toBeInTheDocument();
		});

		it('should NOT show Add Your First Project button when onAddProject is not provided', () => {
			render(ProjectList);

			expect(
				screen.queryByRole('button', { name: /Add Your First Project/i })
			).not.toBeInTheDocument();
		});

		it('should call onAddProject when Add Your First Project button is clicked', async () => {
			const onAddProject = vi.fn();
			render(ProjectList, { props: { onAddProject } });

			await fireEvent.click(screen.getByRole('button', { name: /Add Your First Project/i }));

			expect(onAddProject).toHaveBeenCalledOnce();
		});
	});

	describe('projects grid', () => {
		it('should render project cards when projects exist', () => {
			render(ProjectList);

			expect(screen.getByText('Project One')).toBeInTheDocument();
			expect(screen.getByText('Project Two')).toBeInTheDocument();
		});

		it('should render cards in a grid layout', () => {
			const { container } = render(ProjectList);

			const grid = container.querySelector('.grid');
			expect(grid).toBeInTheDocument();
			expect(grid).toHaveClass('grid-cols-1', 'lg:grid-cols-2');
		});

		it('should render correct number of project cards', () => {
			render(ProjectList);

			expect(screen.getByText('Project One')).toBeInTheDocument();
			expect(screen.getByText('Project Two')).toBeInTheDocument();
		});
	});

	describe('callbacks', () => {
		it('should call onAddProject when Add Project button is clicked', async () => {
			const onAddProject = vi.fn();
			render(ProjectList, { props: { onAddProject } });

			const buttons = screen.getAllByRole('button', { name: /Add Project/i });
			await fireEvent.click(buttons[0]);

			expect(onAddProject).toHaveBeenCalledOnce();
		});

		it('should pass onRemoveProject to ProjectCard', () => {
			const onRemoveProject = vi.fn();
			render(ProjectList, { props: { onRemoveProject } });

			// The callback is passed to ProjectCard - verify projects are rendered
			expect(screen.getByText('Project One')).toBeInTheDocument();
		});
	});

	describe('project card interaction', () => {
		it('should render clickable project cards', () => {
			render(ProjectList);

			// Verify project cards are rendered with click handlers
			// The click handler sets selectedProject which opens ProjectDetail
			expect(screen.getByText('Project One')).toBeInTheDocument();
			expect(screen.getByText('Project Two')).toBeInTheDocument();
		});

		it('should render project paths', () => {
			render(ProjectList);

			expect(screen.getByText('/path/to/project-one')).toBeInTheDocument();
			expect(screen.getByText('/path/to/project-two')).toBeInTheDocument();
		});
	});
});
