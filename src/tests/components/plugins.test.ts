import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		MARKETPLACE_SOURCE_TYPES: actual.MARKETPLACE_SOURCE_TYPES ?? [
			{ value: 'github', label: 'GitHub' },
			{ value: 'git', label: 'Git' },
			{ value: 'url', label: 'URL' },
			{ value: 'npm', label: 'NPM' },
			{ value: 'file', label: 'File' },
			{ value: 'directory', label: 'Directory' },
			{ value: 'hostPattern', label: 'Host Pattern' }
		]
	};
});

describe('PluginListEditor Component', () => {
	let PluginListEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: [],
		enabledPlugins: { 'my-plugin': true, 'disabled-plugin': false }
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/plugins/PluginListEditor.svelte');
		PluginListEditor = mod.default;
	});

	it('should render heading', () => {
		render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Enabled Plugins')).toBeInTheDocument();
	});

	it('should render description', () => {
		render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText(/Configure which plugins are enabled/)).toBeInTheDocument();
	});

	it('should show existing plugins', () => {
		render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('my-plugin')).toBeInTheDocument();
		expect(screen.getByText('disabled-plugin')).toBeInTheDocument();
	});

	it('should show empty state when no plugins', () => {
		render(PluginListEditor, {
			props: { settings: { ...mockSettings, enabledPlugins: undefined } as any, onsave: vi.fn() }
		});
		expect(screen.getByText('No plugins configured')).toBeInTheDocument();
	});

	it('should show plugin name input', () => {
		render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByPlaceholderText('Plugin name')).toBeInTheDocument();
	});

	it('should call onsave on save', async () => {
		const onsave = vi.fn();
		render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Plugins'));
		expect(onsave).toHaveBeenCalledOnce();
	});

	it('should have mode select for each plugin', () => {
		const { container } = render(PluginListEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const selects = container.querySelectorAll('select');
		expect(selects.length).toBe(2);
	});

	it('should show tools input when mode is tools', () => {
		const settings = {
			...mockSettings,
			enabledPlugins: { 'tool-plugin': ['read', 'write'] }
		};
		render(PluginListEditor, {
			props: { settings: settings as any, onsave: vi.fn() }
		});
		expect(screen.getByPlaceholderText('tool1, tool2, tool3')).toBeInTheDocument();
	});
});

describe('MarketplaceEditor Component', () => {
	let MarketplaceEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: [],
		extraKnownMarketplaces: undefined
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/plugins/MarketplaceEditor.svelte');
		MarketplaceEditor = mod.default;
	});

	it('should render heading', () => {
		render(MarketplaceEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Extra Marketplaces')).toBeInTheDocument();
	});

	it('should show empty state', () => {
		render(MarketplaceEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText(/No extra marketplaces/)).toBeInTheDocument();
	});

	it('should show Add Marketplace button', () => {
		render(MarketplaceEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Add Marketplace')).toBeInTheDocument();
	});

	it('should show Save Marketplaces button', () => {
		render(MarketplaceEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Save Marketplaces')).toBeInTheDocument();
	});

	it('should render existing marketplaces', () => {
		const settings = {
			...mockSettings,
			extraKnownMarketplaces: {
				'my-market': { source: { source: 'github', repo: 'owner/repo' } }
			}
		};
		render(MarketplaceEditor, {
			props: { settings: settings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('my-market')).toBeInTheDocument();
		expect(screen.getByText('owner/repo')).toBeInTheDocument();
	});
});

describe('MarketplaceSourceForm Component', () => {
	let MarketplaceSourceForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/plugins/MarketplaceSourceForm.svelte');
		MarketplaceSourceForm = mod.default;
	});

	it('should render form fields', () => {
		render(MarketplaceSourceForm, {
			props: { onsave: vi.fn(), oncancel: vi.fn() }
		});
		expect(screen.getByText('Marketplace Name')).toBeInTheDocument();
		expect(screen.getByText('Source Type')).toBeInTheDocument();
	});

	it('should show github fields by default', () => {
		render(MarketplaceSourceForm, {
			props: { onsave: vi.fn(), oncancel: vi.fn() }
		});
		expect(screen.getByPlaceholderText('owner/repo')).toBeInTheDocument();
	});

	it('should show Cancel and Add Marketplace buttons', () => {
		render(MarketplaceSourceForm, {
			props: { onsave: vi.fn(), oncancel: vi.fn() }
		});
		expect(screen.getByText('Cancel')).toBeInTheDocument();
		expect(screen.getByText('Add Marketplace')).toBeInTheDocument();
	});

	it('should show Update text when editing', () => {
		render(MarketplaceSourceForm, {
			props: {
				name: 'existing',
				definition: { source: { source: 'github', repo: 'foo/bar' } },
				onsave: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Update Marketplace')).toBeInTheDocument();
	});

	it('should show install location field', () => {
		render(MarketplaceSourceForm, {
			props: { onsave: vi.fn(), oncancel: vi.fn() }
		});
		expect(screen.getByText(/Install Location/)).toBeInTheDocument();
	});
});

describe('Plugins index.ts exports', () => {
	let pluginExports: any;

	beforeAll(async () => {
		pluginExports = await import('$lib/components/plugins');
	});

	it('should export all components', () => {
		expect(pluginExports.PluginListEditor).toBeDefined();
		expect(pluginExports.MarketplaceEditor).toBeDefined();
		expect(pluginExports.MarketplaceSourceForm).toBeDefined();
	});
});
