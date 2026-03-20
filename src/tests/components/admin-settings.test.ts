import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import type { SvelteComponent } from 'svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		MANAGED_SETTINGS_FIELDS: actual.MANAGED_SETTINGS_FIELDS ?? [
			{ key: 'disableCustomApiKey', label: 'Disable Custom API Key', description: 'Prevent users from setting custom API keys', type: 'boolean' },
			{ key: 'apiKeyBlocked', label: 'API Key Blocked', description: 'Block specific API key patterns', type: 'stringArray' }
		]
	};
});

describe('ManagedSettingsViewer Component', () => {
	let ManagedSettingsViewer: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/admin-settings/ManagedSettingsViewer.svelte');
		ManagedSettingsViewer = mod.default;
	});

	it('should show not found state when file does not exist', () => {
		render(ManagedSettingsViewer, {
			props: {
				info: {
					exists: false,
					filePath: '/etc/claude/managed_settings.json',
					settings: null
				}
			}
		});
		expect(screen.getByText('No Managed Settings Found')).toBeInTheDocument();
		expect(screen.getByText('Not Found')).toBeInTheDocument();
	});

	it('should show found badge when file exists', () => {
		render(ManagedSettingsViewer, {
			props: {
				info: {
					exists: true,
					filePath: '/etc/claude/managed_settings.json',
					settings: { scope: 'managed', availableModels: [] }
				}
			}
		});
		expect(screen.getByText('Found')).toBeInTheDocument();
		expect(screen.getByText('Managed-Only Policies')).toBeInTheDocument();
	});

	it('should display file path', () => {
		render(ManagedSettingsViewer, {
			props: {
				info: {
					exists: false,
					filePath: '/etc/claude/managed_settings.json',
					settings: null
				}
			}
		});
		const pathElements = screen.getAllByText('/etc/claude/managed_settings.json');
		expect(pathElements.length).toBeGreaterThan(0);
	});
});

describe('Admin-settings index.ts exports', () => {
	let adminExports: any;

	beforeAll(async () => {
		adminExports = await import('$lib/components/admin-settings');
	});

	it('should export ManagedSettingsViewer', () => {
		expect(adminExports.ManagedSettingsViewer).toBeDefined();
	});
});
