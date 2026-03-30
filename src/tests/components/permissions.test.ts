import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	permissionLibrary: {
		isLoading: false,
		permissions: {},
		defaultMode: 'ask',
		load: vi.fn(),
		save: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

describe('PermissionRuleList Component', () => {
	let PermissionRuleList: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/PermissionRuleList.svelte');
		PermissionRuleList = mod.default;
	});

	it('should render deny category heading', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: [], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('Deny')).toBeInTheDocument();
	});

	it('should render allow category heading', () => {
		render(PermissionRuleList, {
			props: { category: 'allow', rules: [], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('Allow')).toBeInTheDocument();
	});

	it('should render ask category heading', () => {
		render(PermissionRuleList, {
			props: { category: 'ask', rules: [], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('Ask')).toBeInTheDocument();
	});

	it('should show rule count badge when rules present', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: ['Bash(rm:*)'], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('1')).toBeInTheDocument();
	});

	it('should parse and display tool name from rule', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: ['Bash(rm:*)'], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getAllByText('Bash').length).toBeGreaterThan(0);
	});

	it('should parse and display specifier from rule', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: ['Bash(rm:*)'], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('(rm:*)')).toBeInTheDocument();
	});

	it('should show empty state when no rules', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: [], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('No deny rules configured')).toBeInTheDocument();
	});

	it('should show Add button', () => {
		render(PermissionRuleList, {
			props: { category: 'deny', rules: [], onremove: vi.fn(), onadd: vi.fn(), onreorder: vi.fn() }
		});
		expect(screen.getByText('Add Deny Rule')).toBeInTheDocument();
	});

	it('should call onadd when Add button clicked', async () => {
		const onadd = vi.fn();
		render(PermissionRuleList, {
			props: { category: 'allow', rules: [], onremove: vi.fn(), onadd, onreorder: vi.fn() }
		});
		await fireEvent.click(screen.getByText('Add Allow Rule'));
		expect(onadd).toHaveBeenCalledOnce();
	});
});

describe('PermissionRuleForm Component', () => {
	let PermissionRuleForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/PermissionRuleForm.svelte');
		PermissionRuleForm = mod.default;
	});

	it('should render dialog with category label', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Add Allow Rule')).toBeInTheDocument();
	});

	it('should render deny rule dialog', () => {
		render(PermissionRuleForm, {
			props: { category: 'deny', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Add Deny Rule')).toBeInTheDocument();
	});

	it('should render ask rule dialog', () => {
		render(PermissionRuleForm, {
			props: { category: 'ask', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Add Ask Rule')).toBeInTheDocument();
	});

	it('should show Builder and Raw mode buttons', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Builder')).toBeInTheDocument();
		expect(screen.getByText('Raw')).toBeInTheDocument();
	});

	it('should show Tool selector in builder mode', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Tool')).toBeInTheDocument();
	});

	it('should show Specifier field in builder mode', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText(/Specifier/)).toBeInTheDocument();
	});

	it('should show Preview section', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Preview')).toBeInTheDocument();
	});

	it('should show Cancel and Add Rule buttons', () => {
		render(PermissionRuleForm, {
			props: { category: 'allow', onsubmit: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Cancel')).toBeInTheDocument();
		expect(screen.getByText('Add Rule')).toBeInTheDocument();
	});
});

describe('PermissionScopeSelector Component', () => {
	let PermissionScopeSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/PermissionScopeSelector.svelte');
		PermissionScopeSelector = mod.default;
	});

	it('should render all scope buttons', () => {
		render(PermissionScopeSelector, {
			props: { selectedScope: 'user', permissions: null, hasProject: true, onselect: vi.fn() }
		});
		expect(screen.getByText('User')).toBeInTheDocument();
		expect(screen.getByText('Project')).toBeInTheDocument();
		expect(screen.getByText('Local')).toBeInTheDocument();
	});

	it('should disable non-user scopes when no project', () => {
		render(PermissionScopeSelector, {
			props: { selectedScope: 'user', permissions: null, hasProject: false, onselect: vi.fn() }
		});
		const buttons = screen.getAllByRole('button');
		// Project and Local buttons should be disabled
		const projectBtn = buttons.find(b => b.textContent?.includes('Project'));
		expect(projectBtn).toBeDisabled();
	});

	it('should show rule count when permissions have rules', () => {
		const permissions = {
			user: { allow: ['Bash'], deny: [], ask: [] },
			project: null,
			local: null
		};
		render(PermissionScopeSelector, {
			props: { selectedScope: 'user', permissions, hasProject: true, onselect: vi.fn() }
		});
		expect(screen.getByText('1')).toBeInTheDocument();
	});

	it('should call onselect when scope clicked', async () => {
		const onselect = vi.fn();
		render(PermissionScopeSelector, {
			props: { selectedScope: 'user', permissions: null, hasProject: true, onselect }
		});
		await fireEvent.click(screen.getByText('Project'));
		expect(onselect).toHaveBeenCalledWith('project');
	});
});

describe('DefaultModeSelector Component', () => {
	let DefaultModeSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/DefaultModeSelector.svelte');
		DefaultModeSelector = mod.default;
	});

	it('should render Default Mode label', () => {
		render(DefaultModeSelector, {
			props: { value: undefined, onchange: vi.fn() }
		});
		expect(screen.getByText('Default Mode')).toBeInTheDocument();
	});

	it('should render a select element', () => {
		const { container } = render(DefaultModeSelector, {
			props: { value: undefined, onchange: vi.fn() }
		});
		expect(container.querySelector('select')).toBeInTheDocument();
	});
});

describe('AdditionalDirectoriesEditor Component', () => {
	let AdditionalDirectoriesEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/AdditionalDirectoriesEditor.svelte');
		AdditionalDirectoriesEditor = mod.default;
	});

	it('should render heading', () => {
		render(AdditionalDirectoriesEditor, {
			props: { directories: [], onchange: vi.fn() }
		});
		expect(screen.getByText('Additional Directories')).toBeInTheDocument();
	});

	it('should show Add button when not adding', () => {
		render(AdditionalDirectoriesEditor, {
			props: { directories: [], onchange: vi.fn() }
		});
		expect(screen.getByText('Add')).toBeInTheDocument();
	});

	it('should show empty state when no directories', () => {
		render(AdditionalDirectoriesEditor, {
			props: { directories: [], onchange: vi.fn() }
		});
		expect(screen.getByText('No additional directories configured')).toBeInTheDocument();
	});

	it('should render existing directories', () => {
		render(AdditionalDirectoriesEditor, {
			props: { directories: ['/usr/local', '/tmp'], onchange: vi.fn() }
		});
		expect(screen.getByText('/usr/local')).toBeInTheDocument();
		expect(screen.getByText('/tmp')).toBeInTheDocument();
	});
});

describe('PermissionMergedView Component', () => {
	let PermissionMergedView: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/PermissionMergedView.svelte');
		PermissionMergedView = mod.default;
	});

	it('should render heading', () => {
		render(PermissionMergedView, {
			props: { rules: [], onclose: vi.fn() }
		});
		expect(screen.getByText('Merged Permissions View')).toBeInTheDocument();
	});

	it('should show empty state when no rules', () => {
		render(PermissionMergedView, {
			props: { rules: [], onclose: vi.fn() }
		});
		expect(screen.getByText('No permission rules configured')).toBeInTheDocument();
	});

	it('should render rules with categories and scopes', () => {
		const rules = [
			{ rule: 'Bash(rm:*)', category: 'deny' as const, scope: 'user' },
			{ rule: 'Read', category: 'allow' as const, scope: 'project' }
		];
		render(PermissionMergedView, { props: { rules, onclose: vi.fn() } });
		expect(screen.getByText('Bash(rm:*)')).toBeInTheDocument();
		expect(screen.getByText('deny')).toBeInTheDocument();
		expect(screen.getByText('allow')).toBeInTheDocument();
		expect(screen.getByText('user')).toBeInTheDocument();
		expect(screen.getByText('project')).toBeInTheDocument();
	});

	it('should render Close button', () => {
		render(PermissionMergedView, {
			props: { rules: [], onclose: vi.fn() }
		});
		expect(screen.getByText('Close')).toBeInTheDocument();
	});

	it('should call onclose when Close clicked', async () => {
		const onclose = vi.fn();
		render(PermissionMergedView, { props: { rules: [], onclose } });
		await fireEvent.click(screen.getByText('Close'));
		expect(onclose).toHaveBeenCalledOnce();
	});
});

describe('PermissionTemplatePanel Component', () => {
	let PermissionTemplatePanel: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/permissions/PermissionTemplatePanel.svelte');
		PermissionTemplatePanel = mod.default;
	});

	it('should render heading', () => {
		render(PermissionTemplatePanel, {
			props: { templates: [], onApply: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('Rule Templates')).toBeInTheDocument();
	});

	it('should show empty state when no templates', () => {
		render(PermissionTemplatePanel, {
			props: { templates: [], onApply: vi.fn(), onclose: vi.fn() }
		});
		expect(screen.getByText('No templates available')).toBeInTheDocument();
	});

	it('should render templates grouped by category', () => {
		const templates = [
			{ name: 'Block rm', rule: 'Bash(rm:*)', category: 'deny' as const, description: 'Block rm commands' },
			{ name: 'Allow Read', rule: 'Read', category: 'allow' as const }
		];
		render(PermissionTemplatePanel, { props: { templates, onApply: vi.fn(), onclose: vi.fn() } });
		expect(screen.getByText('Block rm')).toBeInTheDocument();
		expect(screen.getByText('Allow Read')).toBeInTheDocument();
		expect(screen.getByText('Block rm commands')).toBeInTheDocument();
	});

	it('should call onApply when template clicked', async () => {
		const onApply = vi.fn();
		const templates = [
			{ name: 'Block rm', rule: 'Bash(rm:*)', category: 'deny' as const }
		];
		render(PermissionTemplatePanel, { props: { templates, onApply, onclose: vi.fn() } });
		await fireEvent.click(screen.getByText('Block rm'));
		expect(onApply).toHaveBeenCalledOnce();
	});
});

describe('Permission Tool Names Expansion', () => {
	it('should include Bash tool type', async () => {
		const { PERMISSION_TOOL_NAMES } = await import('$lib/types/permission');
		const bash = PERMISSION_TOOL_NAMES.find(t => t.value === 'Bash');
		expect(bash).toBeDefined();
		expect(bash!.hint).toContain('Bash');
	});

	it('should include Task tool type', async () => {
		const { PERMISSION_TOOL_NAMES } = await import('$lib/types/permission');
		const task = PERMISSION_TOOL_NAMES.find(t => t.value === 'Task');
		expect(task).toBeDefined();
	});

	it('should include WebFetch tool type', async () => {
		const { PERMISSION_TOOL_NAMES } = await import('$lib/types/permission');
		const webFetch = PERMISSION_TOOL_NAMES.find(t => t.value === 'WebFetch');
		expect(webFetch).toBeDefined();
		expect(webFetch!.hint).toContain('WebFetch');
	});
});

describe('Permissions index.ts exports', () => {
	let permExports: any;

	beforeAll(async () => {
		permExports = await import('$lib/components/permissions');
	});

	it('should export all components', () => {
		expect(permExports.PermissionRuleList).toBeDefined();
		expect(permExports.PermissionRuleForm).toBeDefined();
		expect(permExports.PermissionScopeSelector).toBeDefined();
		expect(permExports.PermissionTemplatePanel).toBeDefined();
		expect(permExports.PermissionMergedView).toBeDefined();
		expect(permExports.DefaultModeSelector).toBeDefined();
		expect(permExports.AdditionalDirectoriesEditor).toBeDefined();
	});
});
