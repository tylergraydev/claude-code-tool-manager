import type { Page } from '@playwright/test';

/**
 * Mock data for E2E tests
 */
export const mockData = {
	mcps: [
		{
			id: 1,
			name: 'filesystem',
			description: 'Access to local filesystem operations',
			type: 'stdio',
			command: 'npx',
			args: ['-y', '@anthropic-ai/mcp-server-filesystem', '/Users'],
			url: null,
			headers: null,
			env: null,
			icon: null,
			tags: ['file', 'storage'],
			source: 'manual',
			sourcePath: null,
			isEnabledGlobal: true,
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z'
		},
		{
			id: 2,
			name: 'github',
			description: 'GitHub API integration',
			type: 'stdio',
			command: 'npx',
			args: ['-y', '@anthropic-ai/mcp-server-github'],
			url: null,
			headers: null,
			env: { GITHUB_TOKEN: 'test-token' },
			icon: null,
			tags: ['git', 'api'],
			source: 'manual',
			sourcePath: null,
			isEnabledGlobal: false,
			createdAt: '2024-01-02T00:00:00Z',
			updatedAt: '2024-01-02T00:00:00Z'
		},
		{
			id: 3,
			name: 'http-api',
			description: 'HTTP API server',
			type: 'http',
			command: null,
			args: null,
			url: 'https://api.example.com/mcp',
			headers: { Authorization: 'Bearer test' },
			env: null,
			icon: null,
			tags: ['api', 'http'],
			source: 'manual',
			sourcePath: null,
			isEnabledGlobal: false,
			createdAt: '2024-01-03T00:00:00Z',
			updatedAt: '2024-01-03T00:00:00Z'
		}
	],
	projects: [
		{
			id: 1,
			name: 'test-project',
			path: '/Users/test/projects/test-project',
			hasMcpFile: true,
			hasSettingsFile: true,
			lastScannedAt: '2024-01-01T00:00:00Z',
			editorType: 'claude_code',
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z',
			assignedMcps: [
				{
					id: 1,
					mcpId: 1,
					mcp: null, // Will be populated in handler
					isEnabled: true,
					envOverrides: null,
					displayOrder: 0
				}
			]
		},
		{
			id: 2,
			name: 'another-project',
			path: '/Users/test/projects/another-project',
			hasMcpFile: false,
			hasSettingsFile: false,
			lastScannedAt: null,
			editorType: 'claude_code',
			createdAt: '2024-01-02T00:00:00Z',
			updatedAt: '2024-01-02T00:00:00Z',
			assignedMcps: []
		}
	],
	globalMcps: [
		{
			id: 1,
			mcpId: 1,
			mcp: null, // Will be populated in handler
			isEnabled: true,
			envOverrides: null
		}
	],
	skills: [
		{
			id: 1,
			name: 'commit',
			type: 'command',
			description: 'Create a git commit with a descriptive message',
			content: 'Create a commit with a descriptive message...',
			source: 'manual',
			sourcePath: null,
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z'
		}
	],
	globalSkills: [],
	subagents: [
		{
			id: 1,
			name: 'code-reviewer',
			type: 'agent',
			description: 'Reviews code for issues and improvements',
			content: 'Review the code and provide feedback...',
			model: 'sonnet',
			source: 'manual',
			sourcePath: null,
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z'
		}
	],
	globalSubAgents: [],
	hooks: [],
	repos: [],
	profiles: [
		{
			id: 1,
			name: 'default-profile',
			description: 'Default profile configuration',
			icon: null,
			isActive: true,
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z'
		}
	],
	activeProfile: {
		id: 1,
		name: 'default-profile',
		description: 'Default profile configuration',
		icon: null,
		isActive: true,
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z'
	},
	statuslines: [] as any[],
	activeStatusLine: null as any,
	commands: [
		{
			id: 1,
			name: 'test-command',
			description: 'A test command',
			content: 'echo hello',
			source: 'manual',
			isFavorite: false,
			createdAt: '2024-01-01T00:00:00Z',
			updatedAt: '2024-01-01T00:00:00Z'
		}
	],
	appSettings: {
		defaultEditor: 'claude_code'
	}
};

/**
 * Inject Tauri API mocks into the page
 */
export async function mockTauriApi(page: Page) {
	await page.addInitScript(() => {
		// Mock data (will be available as window.__MOCK_DATA__)
		const mockData = {
			mcps: [
				{
					id: 1,
					name: 'filesystem',
					description: 'Access to local filesystem operations',
					type: 'stdio',
					command: 'npx',
					args: ['-y', '@anthropic-ai/mcp-server-filesystem', '/Users'],
					url: null,
					headers: null,
					env: null,
					icon: null,
					tags: ['file', 'storage'],
					source: 'manual',
					sourcePath: null,
					isEnabledGlobal: true,
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z'
				},
				{
					id: 2,
					name: 'github',
					description: 'GitHub API integration',
					type: 'stdio',
					command: 'npx',
					args: ['-y', '@anthropic-ai/mcp-server-github'],
					url: null,
					headers: null,
					env: { GITHUB_TOKEN: 'test-token' },
					icon: null,
					tags: ['git', 'api'],
					source: 'manual',
					sourcePath: null,
					isEnabledGlobal: false,
					createdAt: '2024-01-02T00:00:00Z',
					updatedAt: '2024-01-02T00:00:00Z'
				},
				{
					id: 3,
					name: 'http-api',
					description: 'HTTP API server',
					type: 'http',
					command: null,
					args: null,
					url: 'https://api.example.com/mcp',
					headers: { Authorization: 'Bearer test' },
					env: null,
					icon: null,
					tags: ['api', 'http'],
					source: 'manual',
					sourcePath: null,
					isEnabledGlobal: false,
					createdAt: '2024-01-03T00:00:00Z',
					updatedAt: '2024-01-03T00:00:00Z'
				}
			],
			projects: [
				{
					id: 1,
					name: 'test-project',
					path: '/Users/test/projects/test-project',
					hasMcpFile: true,
					hasSettingsFile: true,
					lastScannedAt: '2024-01-01T00:00:00Z',
					editorType: 'claude_code',
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z',
					assignedMcps: []
				},
				{
					id: 2,
					name: 'another-project',
					path: '/Users/test/projects/another-project',
					hasMcpFile: false,
					hasSettingsFile: false,
					lastScannedAt: null,
					editorType: 'claude_code',
					createdAt: '2024-01-02T00:00:00Z',
					updatedAt: '2024-01-02T00:00:00Z',
					assignedMcps: []
				}
			],
			globalMcps: [],
			skills: [
				{
					id: 1,
					name: 'commit',
					type: 'command',
					description: 'Create a git commit with a descriptive message',
					content: 'Create a commit with a descriptive message...',
					source: 'manual',
					sourcePath: null,
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z'
				}
			],
			globalSkills: [],
			subagents: [
				{
					id: 1,
					name: 'code-reviewer',
					type: 'agent',
					description: 'Reviews code for issues and improvements',
					content: 'Review the code and provide feedback...',
					model: 'sonnet',
					source: 'manual',
					sourcePath: null,
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z'
				}
			],
			globalSubAgents: [],
			hooks: [],
			repos: [],
			profiles: [
				{
					id: 1,
					name: 'default-profile',
					description: 'Default profile configuration',
					icon: null,
					isActive: true,
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z'
				}
			],
			activeProfile: {
				id: 1,
				name: 'default-profile',
				description: 'Default profile configuration',
				icon: null,
				isActive: true,
				createdAt: '2024-01-01T00:00:00Z',
				updatedAt: '2024-01-01T00:00:00Z'
			},
			statuslines: [],
			activeStatusLine: null,
			commands: [
				{
					id: 1,
					name: 'test-command',
					description: 'A test command',
					content: 'echo hello',
					source: 'manual',
					isFavorite: false,
					createdAt: '2024-01-01T00:00:00Z',
					updatedAt: '2024-01-01T00:00:00Z'
				}
			]
		};

		// Track invoke calls for assertions
		(window as any).__INVOKE_CALLS__ = [];

		// Create mock invoke function
		const mockInvoke = async (cmd: string, args?: any): Promise<any> => {
			// Log the call
			(window as any).__INVOKE_CALLS__.push({ cmd, args, timestamp: Date.now() });
			console.log(`[Tauri Mock] invoke('${cmd}')`, args);

			// Return mock data based on command
			switch (cmd) {
				// MCP commands
				case 'get_all_mcps':
					return mockData.mcps;
				case 'create_mcp':
					const newMcp = { ...args.mcp, id: Date.now(), createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
					mockData.mcps.push(newMcp);
					return newMcp;
				case 'update_mcp':
					return { ...mockData.mcps.find(m => m.id === args.id), ...args.mcp };
				case 'delete_mcp':
					mockData.mcps = mockData.mcps.filter(m => m.id !== args.id);
					return null;
				case 'duplicate_mcp':
					const orig = mockData.mcps.find(m => m.id === args.id);
					const dup = { ...orig, id: Date.now(), name: `${orig?.name} (copy)` };
					mockData.mcps.push(dup);
					return dup;
				case 'toggle_global_mcp':
					return null;

				// Project commands
				case 'get_all_projects':
					return mockData.projects;
				case 'get_global_mcps':
					return mockData.globalMcps;
				case 'add_project':
					const newProj = { ...args.project, id: Date.now(), assignedMcps: [], createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
					mockData.projects.push(newProj);
					return newProj;
				case 'remove_project':
					mockData.projects = mockData.projects.filter(p => p.id !== args.id);
					return null;
				case 'browse_for_project':
					return '/Users/test/projects/new-project';
				case 'assign_mcp_to_project':
				case 'remove_mcp_from_project':
				case 'toggle_project_mcp':
				case 'sync_project_config':
				case 'add_global_mcp':
				case 'remove_global_mcp':
				case 'toggle_global_mcp_assignment':
				case 'sync_global_config':
				case 'scan_claude_directory':
				case 'update_project_editor_type':
					return null;

				// Skill commands
				case 'get_all_skills':
					return mockData.skills;
				case 'get_global_skills':
					return mockData.globalSkills;
				case 'delete_skill':
				case 'add_global_skill':
				case 'remove_global_skill':
				case 'toggle_global_skill':
				case 'assign_skill_to_project':
				case 'remove_skill_from_project':
				case 'toggle_project_skill':
				case 'delete_skill_file':
					return null;

				// SubAgent commands
				case 'get_all_subagents':
					return mockData.subagents;
				case 'get_global_subagents':
					return mockData.globalSubAgents;
				case 'delete_subagent':
				case 'add_global_subagent':
				case 'remove_global_subagent':
				case 'toggle_global_subagent':
				case 'assign_subagent_to_project':
				case 'remove_subagent_from_project':
				case 'toggle_project_subagent':
					return null;

				// Hook commands
				case 'get_all_hooks':
					return mockData.hooks;
				case 'get_global_hooks':
					return [];
				case 'seed_hook_templates':
				case 'delete_hook':
				case 'add_global_hook':
				case 'remove_global_hook':
				case 'toggle_global_hook':
				case 'assign_hook_to_project':
				case 'remove_hook_from_project':
				case 'toggle_project_hook':
					return null;

				// Profile commands
				case 'get_all_profiles':
					return mockData.profiles;
				case 'get_active_profile':
					return mockData.activeProfile;
				case 'create_profile':
					return { ...args, id: Date.now(), createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
				case 'update_profile':
				case 'delete_profile':
				case 'duplicate_profile':
				case 'activate_profile':
				case 'deactivate_profile':
					return null;

				// Status Line commands
				case 'get_all_statuslines':
					return mockData.statuslines;
				case 'get_active_statusline':
					return mockData.activeStatusLine;
				case 'create_statusline':
					return { ...args, id: Date.now(), createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
				case 'update_statusline':
				case 'delete_statusline':
				case 'duplicate_statusline':
				case 'activate_statusline':
				case 'deactivate_statusline':
				case 'get_statusline_gallery':
				case 'import_statusline_from_gallery':
					return null;

				// Command commands
				case 'get_all_commands':
					return mockData.commands;
				case 'create_command':
					return { ...args, id: Date.now(), createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
				case 'update_command':
				case 'delete_command':
				case 'duplicate_command':
					return null;

				// Sound commands
				case 'get_all_sounds':
					return [];
				case 'get_all_sound_notifications':
					return [];
				case 'play_sound':
				case 'stop_sound':
					return null;

				// Repo/marketplace commands
				case 'get_all_repos':
					return mockData.repos;
				case 'seed_default_repos':
				case 'remove_repo':
				case 'toggle_repo':
				case 'reset_repos_to_defaults':
					return null;

				// Settings commands
				case 'get_app_settings':
					return { defaultEditor: 'claude_code' };
				case 'update_app_settings':
				case 'open_config_file':
				case 'backup_configs':
					return null;

				// Debug commands
				case 'enable_debug_mode':
					return '/path/to/debug.log';
				case 'disable_debug_mode':
				case 'is_debug_mode_enabled':
					return false;
				case 'get_debug_log_path':
					return null;
				case 'open_logs_folder':
				case 'write_frontend_log':
				case 'write_invoke_log':
					return null;

				// Claude JSON commands
				case 'toggle_mcp_in_claude_json':
				case 'remove_mcp_from_claude_json':
				case 'remove_global_mcp_from_claude_json':
					return null;

				// MCP testing
				case 'test_mcp':
				case 'test_mcp_config':
					return {
						success: true,
						serverInfo: { name: 'test-server', version: '1.0.0' },
						tools: [{ name: 'test-tool', description: 'A test tool', inputSchema: null }],
						resourcesSupported: false,
						promptsSupported: false,
						error: null,
						responseTimeMs: 100
					};

				// What's new
				case 'get_whats_new_for_version':
					return null;
				case 'mark_whats_new_seen':
				case 'get_last_seen_version':
					return null;

				default:
					console.warn(`[Tauri Mock] Unhandled command: ${cmd}`);
					return null;
			}
		};

		// Define __TAURI_INTERNALS__ for Tauri v2
		Object.defineProperty(window, '__TAURI_INTERNALS__', {
			value: {
				invoke: mockInvoke,
				transformCallback: (callback: any) => callback,
				convertFileSrc: (src: string) => src
			},
			writable: true,
			configurable: true
		});

		// Also define __TAURI__ for compatibility
		Object.defineProperty(window, '__TAURI__', {
			value: {
				invoke: mockInvoke
			},
			writable: true,
			configurable: true
		});
	});
}

/**
 * Get the list of invoke calls made during the test
 */
export async function getInvokeCalls(page: Page): Promise<Array<{ cmd: string; args?: any; timestamp: number }>> {
	return await page.evaluate(() => (window as any).__INVOKE_CALLS__ || []);
}

/**
 * Clear the invoke call log
 */
export async function clearInvokeCalls(page: Page): Promise<void> {
	await page.evaluate(() => {
		(window as any).__INVOKE_CALLS__ = [];
	});
}
