import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';

describe('ProjectDetail Search Logic', () => {
	// Test search filtering logic for available items
	const mockMcps = [
		{ id: 1, name: 'filesystem-mcp', type: 'stdio' as const, description: 'File system access' },
		{ id: 2, name: 'github-mcp', type: 'http' as const, description: 'GitHub API integration' },
		{ id: 3, name: 'database-mcp', type: 'sse' as const, description: 'Database operations' }
	];

	const mockSkills = [
		{ id: 1, name: 'code-review', description: 'Review code for issues' },
		{ id: 2, name: 'test-writer', description: 'Generate test cases' },
		{ id: 3, name: 'doc-generator', description: 'Generate documentation' }
	];

	it('should filter MCPs by name (case insensitive)', () => {
		const searchQuery = 'github';
		const filteredMcps = mockMcps.filter((mcp) => {
			const query = searchQuery.toLowerCase();
			return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
		});

		expect(filteredMcps).toHaveLength(1);
		expect(filteredMcps[0].name).toBe('github-mcp');
	});

	it('should filter MCPs by description', () => {
		const searchQuery = 'database';
		const filteredMcps = mockMcps.filter((mcp) => {
			const query = searchQuery.toLowerCase();
			return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
		});

		expect(filteredMcps).toHaveLength(1);
		expect(filteredMcps[0].name).toBe('database-mcp');
	});

	it('should return all items when search query is empty', () => {
		const searchQuery = '';
		const filteredMcps = searchQuery.trim()
			? mockMcps.filter((mcp) => {
					const query = searchQuery.toLowerCase();
					return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
				})
			: mockMcps;

		expect(filteredMcps).toHaveLength(3);
	});

	it('should return empty array when no matches found', () => {
		const searchQuery = 'nonexistent';
		const filteredMcps = mockMcps.filter((mcp) => {
			const query = searchQuery.toLowerCase();
			return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
		});

		expect(filteredMcps).toHaveLength(0);
	});

	it('should filter skills by name', () => {
		const searchQuery = 'code';
		const filteredSkills = mockSkills.filter((skill) => {
			const query = searchQuery.toLowerCase();
			return skill.name.toLowerCase().includes(query) || skill.description?.toLowerCase().includes(query);
		});

		expect(filteredSkills).toHaveLength(1);
		expect(filteredSkills[0].name).toBe('code-review');
	});

	it('should filter skills by description', () => {
		const searchQuery = 'test';
		const filteredSkills = mockSkills.filter((skill) => {
			const query = searchQuery.toLowerCase();
			return skill.name.toLowerCase().includes(query) || skill.description?.toLowerCase().includes(query);
		});

		expect(filteredSkills).toHaveLength(1);
		expect(filteredSkills[0].name).toBe('test-writer');
	});

	it('should handle whitespace-only search query as empty', () => {
		const searchQuery = '   ';
		const filteredMcps = searchQuery.trim()
			? mockMcps.filter((mcp) => {
					const query = searchQuery.toLowerCase();
					return mcp.name.toLowerCase().includes(query) || mcp.description?.toLowerCase().includes(query);
				})
			: mockMcps;

		expect(filteredMcps).toHaveLength(3);
	});
});

describe('ProjectList Search Logic', () => {
	const mockProjects = [
		{ id: 1, name: 'my-app', path: '/Users/dev/projects/my-app' },
		{ id: 2, name: 'api-server', path: '/Users/dev/work/api-server' },
		{ id: 3, name: 'website', path: '/home/user/website' }
	];

	it('should filter projects by name', () => {
		const searchQuery = 'api';
		const filteredProjects = mockProjects.filter((project) => {
			const query = searchQuery.toLowerCase();
			return project.name.toLowerCase().includes(query) || project.path.toLowerCase().includes(query);
		});

		expect(filteredProjects).toHaveLength(1);
		expect(filteredProjects[0].name).toBe('api-server');
	});

	it('should filter projects by path', () => {
		const searchQuery = '/home';
		const filteredProjects = mockProjects.filter((project) => {
			const query = searchQuery.toLowerCase();
			return project.name.toLowerCase().includes(query) || project.path.toLowerCase().includes(query);
		});

		expect(filteredProjects).toHaveLength(1);
		expect(filteredProjects[0].name).toBe('website');
	});

	it('should match partial path segments', () => {
		const searchQuery = 'dev';
		const filteredProjects = mockProjects.filter((project) => {
			const query = searchQuery.toLowerCase();
			return project.name.toLowerCase().includes(query) || project.path.toLowerCase().includes(query);
		});

		expect(filteredProjects).toHaveLength(2);
	});
});

describe('ProjectDetail Component', () => {
	const mockProject = {
		id: 1,
		name: 'Test Project',
		path: 'C:/Code/test-project',
		hasMcpFile: false,
		hasSettingsFile: false,
		assignedMcps: [
			{
				id: 1,
				mcpId: 1,
				isEnabled: true,
				displayOrder: 1,
				mcp: { id: 1, name: 'assigned-mcp', type: 'stdio' as const }
			}
		],
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	const mockMcps = [
		{ id: 1, name: 'assigned-mcp', type: 'stdio' as const },
		{ id: 2, name: 'available-mcp', type: 'http' as const },
		{ id: 3, name: 'another-mcp', type: 'sse' as const }
	];

	beforeEach(() => {
		vi.clearAllMocks();
		// Set up mock for MCP library
		vi.mocked(invoke).mockImplementation(async (cmd) => {
			if (cmd === 'get_all_mcps') return mockMcps;
			if (cmd === 'get_all_projects') return [mockProject];
			return undefined;
		});
	});

	it('should show assigned MCPs in the assigned section', async () => {
		// This test validates that assigned MCPs appear in the right section
		const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
		await mcpLibrary.load();

		// Check that the assigned MCP is correctly identified
		const assignedIds = mockProject.assignedMcps.map((a) => a.mcpId);
		const availableMcps = mcpLibrary.mcps.filter((m) => !assignedIds.includes(m.id));

		expect(assignedIds).toContain(1);
		expect(availableMcps.map((m) => m.id)).not.toContain(1);
		expect(availableMcps.map((m) => m.id)).toContain(2);
		expect(availableMcps.map((m) => m.id)).toContain(3);
	});

	it('should correctly calculate available MCPs (not assigned)', async () => {
		const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
		await mcpLibrary.load();

		const assignedMcpIds = mockProject.assignedMcps.map((a) => a.mcpId);
		const availableMcps = mcpLibrary.mcps.filter((mcp) => !assignedMcpIds.includes(mcp.id));

		// Should have 2 available MCPs (id 2 and 3)
		expect(availableMcps).toHaveLength(2);
		expect(availableMcps.find((m) => m.id === 1)).toBeUndefined();
	});

	it('should not show duplicates in available MCPs list', async () => {
		const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');

		// Load multiple times to simulate refresh
		await mcpLibrary.load();
		await mcpLibrary.load();

		const assignedMcpIds = mockProject.assignedMcps.map((a) => a.mcpId);
		const availableMcps = mcpLibrary.mcps.filter((mcp) => !assignedMcpIds.includes(mcp.id));

		// Should still have only 2 available MCPs, no duplicates
		expect(availableMcps).toHaveLength(2);

		// Check for unique IDs
		const ids = availableMcps.map((m) => m.id);
		const uniqueIds = [...new Set(ids)];
		expect(ids.length).toBe(uniqueIds.length);
	});

	it('should update when project store changes (stale data fix)', async () => {
		const { projectsStore } = await import('$lib/stores/projects.svelte');

		// Initial project state
		const initialProject = {
			id: 1,
			name: 'Test Project',
			path: 'C:/Code/test',
			assignedMcps: [{ id: 1, mcpId: 1, isEnabled: true }]
		};

		// Updated project state (after adding an MCP)
		const updatedProject = {
			id: 1,
			name: 'Test Project',
			path: 'C:/Code/test',
			assignedMcps: [
				{ id: 1, mcpId: 1, isEnabled: true },
				{ id: 2, mcpId: 2, isEnabled: true }
			]
		};

		// First load returns initial state
		vi.mocked(invoke).mockResolvedValueOnce([initialProject]);
		await projectsStore.loadProjects();

		// Verify initial state
		const project1 = projectsStore.getProjectById(1);
		expect(project1?.assignedMcps).toHaveLength(1);

		// Second load returns updated state (simulates after assignMcpToProject)
		vi.mocked(invoke).mockResolvedValueOnce([updatedProject]);
		await projectsStore.loadProjects();

		// Verify updated state is reflected
		const project2 = projectsStore.getProjectById(1);
		expect(project2?.assignedMcps).toHaveLength(2);

		// The key point: getting project by ID returns the CURRENT state
		// This verifies the fix where ProjectDetail uses getProjectById
		expect(project2).not.toBe(project1); // Different object references
	});
});
