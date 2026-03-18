import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	claudeSettingsLibrary: {
		isLoading: false,
		error: null,
		settings: null,
		selectedScope: 'user',
		currentScopeSettings: null,
		load: vi.fn().mockResolvedValue(undefined),
		save: vi.fn(),
		setScope: vi.fn()
	},
	projectsStore: {
		projects: [],
		globalMcps: [],
		globalSkills: [],
		globalSubAgents: [],
		globalCommands: [],
		loadProjects: vi.fn().mockResolvedValue(undefined),
		loadGlobalMcps: vi.fn().mockResolvedValue(undefined),
		addGlobalMcp: vi.fn(),
		removeGlobalMcp: vi.fn(),
		updateGlobalMcpEnabled: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	keybindingsLibrary: {
		isLoading: false,
		error: null,
		bindings: {},
		mergedBindings: [],
		filteredByContext: new Map(),
		expandedContexts: new Set(),
		searchQuery: '',
		load: vi.fn().mockResolvedValue(undefined),
		expandAll: vi.fn(),
		toggleContext: vi.fn(),
		setBinding: vi.fn(),
		unbindKey: vi.fn(),
		resetContext: vi.fn(),
		save: vi.fn(),
		setSearch: vi.fn()
	},
	spinnerVerbLibrary: {
		verbs: [],
		mode: 'append',
		isLoading: false,
		load: vi.fn().mockResolvedValue(undefined),
		create: vi.fn(),
		update: vi.fn(),
		delete: vi.fn(),
		setMode: vi.fn(),
		sync: vi.fn()
	},
	whatsNew: {
		showDialog: false
	},
	statuslineLibrary: {
		statuslines: [],
		isLoading: false,
		load: vi.fn()
	},
	mcpLibrary: {
		mcps: [],
		getMcpById: vi.fn()
	},
	skillLibrary: {
		globalSkills: [],
		skills: [],
		isLoading: false,
		load: vi.fn(),
		loadGlobalSkills: vi.fn().mockResolvedValue(undefined)
	},
	subagentLibrary: {
		globalSubAgents: [],
		subagents: [],
		isLoading: false,
		load: vi.fn(),
		loadGlobalSubAgents: vi.fn().mockResolvedValue(undefined)
	},
	commandLibrary: {
		commands: [],
		globalCommands: [],
		isLoading: false,
		load: vi.fn(),
		loadGlobalCommands: vi.fn().mockResolvedValue(undefined)
	},
	debugStore: {
		isEnabled: false,
		log: vi.fn(),
		load: vi.fn().mockResolvedValue(undefined)
	}
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn().mockImplementation((cmd: string) => {
		if (cmd === 'get_mcp_server_config') {
			return Promise.resolve({ enabled: false, port: 23847, autoStart: false });
		}
		if (cmd === 'get_gateway_config') {
			return Promise.resolve({ enabled: false, port: 23848, autoStart: false });
		}
		if (cmd === 'get_backend_info') {
			return Promise.resolve({ version: '1.0.0', databasePath: '/tmp/db' });
		}
		return Promise.resolve(null);
	})
}));

vi.mock('@tauri-apps/api/app', () => ({
	getVersion: vi.fn().mockResolvedValue('3.0.0')
}));

vi.mock('$lib/types', async (importOriginal) => {
	const actual = (await importOriginal()) as any;
	return {
		...actual,
		CLAUDE_SETTINGS_SCOPE_LABELS: actual.CLAUDE_SETTINGS_SCOPE_LABELS ?? {
			user: { label: 'User', description: 'User scope' },
			project: { label: 'Project', description: 'Project scope' },
			local: { label: 'Local', description: 'Local scope' }
		}
	};
});

// ──────────────────────────────────────────────────────────
// SettingsAdminTab
// ──────────────────────────────────────────────────────────
describe('SettingsAdminTab', () => {
	let SettingsAdminTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsAdminTab.svelte');
		SettingsAdminTab = mod.default;
	});

	it('should render loading spinner initially', () => {
		const { container } = render(SettingsAdminTab);
		expect(container.querySelector('.animate-spin')).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsAuthTab
// ──────────────────────────────────────────────────────────
describe('SettingsAuthTab', () => {
	let SettingsAuthTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsAuthTab.svelte');
		SettingsAuthTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsAuthTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsEnvironmentTab
// ──────────────────────────────────────────────────────────
describe('SettingsEnvironmentTab', () => {
	let SettingsEnvironmentTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsEnvironmentTab.svelte');
		SettingsEnvironmentTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsEnvironmentTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsFilesTab
// ──────────────────────────────────────────────────────────
describe('SettingsFilesTab', () => {
	let SettingsFilesTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsFilesTab.svelte');
		SettingsFilesTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsFilesTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsInterfaceTab
// ──────────────────────────────────────────────────────────
describe('SettingsInterfaceTab', () => {
	let SettingsInterfaceTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsInterfaceTab.svelte');
		SettingsInterfaceTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsInterfaceTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsKeybindingsTab
// ──────────────────────────────────────────────────────────
describe('SettingsKeybindingsTab', () => {
	let SettingsKeybindingsTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsKeybindingsTab.svelte');
		SettingsKeybindingsTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsKeybindingsTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsMcpApprovalTab
// ──────────────────────────────────────────────────────────
describe('SettingsMcpApprovalTab', () => {
	let SettingsMcpApprovalTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsMcpApprovalTab.svelte');
		SettingsMcpApprovalTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsMcpApprovalTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsModelsTab
// ──────────────────────────────────────────────────────────
describe('SettingsModelsTab', () => {
	let SettingsModelsTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsModelsTab.svelte');
		SettingsModelsTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsModelsTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsPluginsTab
// ──────────────────────────────────────────────────────────
describe('SettingsPluginsTab', () => {
	let SettingsPluginsTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsPluginsTab.svelte');
		SettingsPluginsTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsPluginsTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsSecurityTab
// ──────────────────────────────────────────────────────────
describe('SettingsSecurityTab', () => {
	let SettingsSecurityTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsSecurityTab.svelte');
		SettingsSecurityTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsSecurityTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsSessionTab
// ──────────────────────────────────────────────────────────
describe('SettingsSessionTab', () => {
	let SettingsSessionTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsSessionTab.svelte');
		SettingsSessionTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsSessionTab);
		expect(document.body).toBeTruthy();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsSpinnerVerbsTab
// ──────────────────────────────────────────────────────────
describe('SettingsSpinnerVerbsTab', () => {
	let SettingsSpinnerVerbsTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsSpinnerVerbsTab.svelte');
		SettingsSpinnerVerbsTab = mod.default;
	});

	it('should render Mode label', () => {
		render(SettingsSpinnerVerbsTab);
		expect(screen.getByText('Mode:')).toBeInTheDocument();
	});

	it('should render mode select with Append and Replace options', () => {
		render(SettingsSpinnerVerbsTab);
		expect(screen.getByText('Append (add to defaults)')).toBeInTheDocument();
		expect(screen.getByText('Replace (use only these)')).toBeInTheDocument();
	});

	it('should render Sync to Settings button', () => {
		render(SettingsSpinnerVerbsTab);
		expect(screen.getByText('Sync to Settings')).toBeInTheDocument();
	});

	it('should render Add Verb button', () => {
		render(SettingsSpinnerVerbsTab);
		expect(screen.getByText('Add Verb')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// SettingsEditorSyncTab (large, complex component)
// ──────────────────────────────────────────────────────────
describe('SettingsEditorSyncTab', () => {
	let SettingsEditorSyncTab: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/settings/tabs/SettingsEditorSyncTab.svelte');
		SettingsEditorSyncTab = mod.default;
	});

	it('should render without crashing', () => {
		render(SettingsEditorSyncTab);
		expect(document.body).toBeTruthy();
	});
});
