import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import ProjectCard from '$lib/components/projects/ProjectCard.svelte';
import type { Project } from '$lib/types';
import { invoke } from '@tauri-apps/api/core';
import { skillLibrary, subagentLibrary } from '$lib/stores';

// Mock the shell plugin
vi.mock('@tauri-apps/plugin-shell', () => ({
	open: vi.fn()
}));

describe('ProjectCard', () => {
	const createMockProject = (overrides: Partial<Project> = {}): Project => ({
		id: 1,
		name: 'Test Project',
		path: '/path/to/project',
		assignedMcps: [],
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01',
		...overrides
	});

	beforeEach(() => {
		vi.clearAllMocks();
		// Mock skill and subagent library methods to return empty arrays
		vi.spyOn(skillLibrary, 'getProjectSkills').mockResolvedValue([]);
		vi.spyOn(subagentLibrary, 'getProjectSubAgents').mockResolvedValue([]);
	});

	describe('rendering', () => {
		it('should render project name', () => {
			render(ProjectCard, { props: { project: createMockProject({ name: 'My Project' }) } });

			expect(screen.getByText('My Project')).toBeInTheDocument();
		});

		it('should render project path', () => {
			render(ProjectCard, {
				props: { project: createMockProject({ path: '/Users/test/my-project' }) }
			});

			expect(screen.getByText('/Users/test/my-project')).toBeInTheDocument();
		});

		it('should have FolderOpen icon', () => {
			const { container } = render(ProjectCard, { props: { project: createMockProject() } });

			const iconContainer = container.querySelector('.bg-amber-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});

	describe('editor type badge', () => {
		it('should show Claude badge by default', () => {
			render(ProjectCard, {
				props: { project: createMockProject({ editorType: undefined }) }
			});

			expect(screen.getByText('Claude')).toBeInTheDocument();
		});

		it('should show OpenCode badge for opencode editor', () => {
			render(ProjectCard, {
				props: { project: createMockProject({ editorType: 'opencode' }) }
			});

			expect(screen.getByText('OpenCode')).toBeInTheDocument();
		});
	});

	describe('.mcp.json badge', () => {
		it('should show .mcp.json badge when hasMcpFile is true', () => {
			render(ProjectCard, {
				props: { project: createMockProject({ hasMcpFile: true }) }
			});

			expect(screen.getByText('.mcp.json')).toBeInTheDocument();
		});

		it('should not show .mcp.json badge when hasMcpFile is false', () => {
			render(ProjectCard, {
				props: { project: createMockProject({ hasMcpFile: false }) }
			});

			expect(screen.queryByText('.mcp.json')).not.toBeInTheDocument();
		});
	});

	describe('MCP counts', () => {
		it('should show count indicators', () => {
			const { container } = render(ProjectCard, {
				props: { project: createMockProject({ assignedMcps: [] }) }
			});

			// Should have 3 count sections (MCPs, Skills, Agents)
			const counts = container.querySelectorAll('.flex.items-center.gap-1\\.5.text-sm');
			expect(counts.length).toBe(3);
		});

		it('should show enabled/total count for MCPs', () => {
			render(ProjectCard, {
				props: {
					project: createMockProject({
						assignedMcps: [
							{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'mcp1', type: 'stdio' } },
							{ id: 2, mcpId: 2, isEnabled: true, mcp: { id: 2, name: 'mcp2', type: 'http' } },
							{ id: 3, mcpId: 3, isEnabled: false, mcp: { id: 3, name: 'mcp3', type: 'sse' } }
						] as Project['assignedMcps']
					})
				}
			});

			// 2 enabled out of 3 total
			expect(screen.getByText('2/3')).toBeInTheDocument();
		});
	});

	describe('click handling', () => {
		it('should be keyboard accessible', () => {
			render(ProjectCard, {
				props: { project: createMockProject() }
			});

			const buttons = screen.getAllByRole('button');
			// Card itself is the first button with tabindex
			const card = buttons.find(btn => btn.getAttribute('tabindex') === '0');
			expect(card).toBeInTheDocument();
		});
	});
});
